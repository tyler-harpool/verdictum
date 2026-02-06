# CourtFlow Implementation — Team Agent Prompt

> **Purpose:** Single source of truth for Claude Code agents implementing CourtFlow into the Verdictum repo.
> **Repo:** `github.com/tyler-harpool/verdictum`
> **Architecture:** Hexagonal (Ports & Adapters) with Spin SDK, compiled to WebAssembly

---

## Context: What Exists

Verdictum is a working Rust/Spin SDK judicial case management API with:
- **350+ HTTP endpoints** across 14 modules (cases, judges, dockets, deadlines, attorneys, sentencing, opinions, orders, parties, PDF generation, multi-tenant admin, feature flags, speedy trial, todos)
- **Hexagonal architecture**: `domain/` → `ports/` → `adapters/` → `handlers/`
- **Multi-tenant isolation** for all 94 federal districts via `RepositoryFactory` + `X-Court-District` header
- **Spin KV storage** with physical file separation per district (`tenant:{district}:entity:{id}` namespace keys)
- **PDF generation** (AO forms, electronic signatures, batch generation)
- **Spin SDK + wasm32-wasip1 target** — compiles to WebAssembly, runs on Fermyon

### Existing Project Structure
```
src/
├── lib.rs                          # Main router
├── error.rs                        # Typed errors
├── utils/
│   └── repository_factory.rs       # Multi-tenant KV store routing
├── domain/                         # Pure business domain
│   ├── criminal_case.rs, judge.rs, docket.rs, deadline.rs
│   ├── features.rs, document.rs, attorney.rs, sentencing.rs
│   ├── party.rs, opinion.rs, order.rs, todo.rs
│   └── mod.rs
├── ports/                          # Repository traits (interfaces)
│   ├── case_repository.rs, judge_repository.rs, docket_repository.rs
│   ├── deadline_repository.rs, feature_repository.rs
│   └── mod.rs
├── adapters/                       # Spin KV implementations
│   ├── spin_kv_case_repository.rs, spin_kv_judge_repository.rs
│   ├── spin_kv_docket_repository.rs, spin_kv_deadline_repository.rs
│   ├── spin_kv_feature_repository.rs
│   └── mod.rs
└── handlers/                       # HTTP endpoints (350+)
    ├── criminal_case.rs, judge.rs, docket.rs, deadline.rs
    ├── features.rs, admin.rs, health.rs, docs.rs
    ├── pdf_hexagonal.rs, attorney.rs, sentencing.rs
    ├── party.rs, opinion.rs, order.rs, todo.rs
    └── mod.rs
config/                             # Spin configs
spin.toml                           # Spin application manifest
Cargo.toml                          # Rust dependencies
runtime-config.toml                 # 94 district KV stores
```

### Key Technology Constraints
- **Spin SDK** — not Axum. HTTP handled via `spin_sdk::http::{IntoResponse, Request, Response}`
- **wasm32-wasip1** — no `std::fs`, no `tokio`, no threads. Spin provides KV, SQLite, outbound HTTP
- **Single binary** — everything compiles to one Wasm component
- **KV storage** — `spin_sdk::key_value::Store` with string keys and byte values (JSON serialized)

---

## What We're Building: CourtFlow Integration

Adding a **rules-as-data compliance engine** to Verdictum. Court rules (FRCP, local rules, standing orders) become structured data objects that a rules engine evaluates at filing time.

### New Capabilities
1. **Rules Engine** — Evaluate structured rule objects against filing metadata → pass/warn/block
2. **FRCP Rule 6 Deadline Engine** — Full day-counting algorithm with weekend/holiday exclusion
3. **Privacy Engine** — FRCP 5.2 PII detection (SSN, DOB, minors, accounts) with auto-block
4. **Filing Pipeline** — Unified submission flow: validate → scan privacy → evaluate rules → compute deadlines → generate NEF
5. **ARWD Rule Pack** — Western District of Arkansas as proof-of-concept (29 local rules, 30 admin procedures, 16 federal rules)

---

## Agent Assignment: 5 Parallel Workstreams

Each agent works independently in a specific area. No agent modifies files outside their designated directories except where explicitly noted.

---

### Agent 1: Domain Model & Rule Types

**Goal:** Add CourtFlow domain types to `src/domain/` following the existing pattern.

**Files to create:**
- `src/domain/rule.rs` — Rule, RuleSource, RuleCategory, RuleAction, TriggerEvent, ComplianceResult
- `src/domain/deadline_calc.rs` — DeadlineComputeRequest, DeadlineResult, FederalHoliday, ServiceMethod
- `src/domain/privacy.rs` — PiiType, PiiMatch, PrivacyScanResult, DocumentRestriction, RestrictedDocType
- `src/domain/filing_pipeline.rs` — FilingSubmission, FilingValidation, FilingContext, ComplianceReport
- `src/domain/nef.rs` — NoticeOfElectronicFiling, NefRecipient, DeliveryMethod

**Files to modify:**
- `src/domain/mod.rs` — Add `pub mod rule; pub mod deadline_calc; pub mod privacy; pub mod filing_pipeline; pub mod nef;`

**Design rules:**
- All entities derive `Debug, Clone, Serialize, Deserialize`
- All enums derive `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`
- Use `serde_json::Value` for extensible condition/parameter fields on Rule
- UUIDs as primary IDs (`uuid::Uuid` or String depending on existing pattern — check `criminal_case.rs` for the pattern used)
- Follow existing naming conventions exactly

**Rule struct (critical):**
```rust
pub struct Rule {
    pub rule_id: String,
    pub jurisdiction_id: String,
    pub rule_source: RuleSource,       // StandingOrder | LocalRule | AdminProcedure | FRCP | Statute
    pub rule_number: String,           // e.g., "7.2(b)", "5.2", "III.A.1"
    pub rule_category: RuleCategory,   // Filing | Deadline | Privacy | Fees | Service | JudgeAssignment
    pub trigger_event: TriggerEvent,   // MotionFiled | ComplaintFiled | DocumentUploaded | etc.
    pub condition: serde_json::Value,  // JSON condition expression
    pub action: RuleAction,            // Enforce | Warn | Validate | Calculate | Block | Notify
    pub parameters: serde_json::Value, // Action-specific params
    pub effective_date: String,        // ISO date
    pub superseded_date: Option<String>,
    pub source_text: String,           // Original rule text for audit
    pub version: u32,
}
```

**RuleSource priority (encode as method):**
```rust
impl RuleSource {
    pub fn priority(&self) -> u8 {
        match self {
            Self::StandingOrder => 1,   // highest
            Self::LocalRule => 2,
            Self::AdminProcedure => 3,
            Self::FRCP => 4,
            Self::Statute => 5,         // lowest
        }
    }
}
```

**Tests:** Serialization roundtrip for each type. Verify RuleSource priority ordering. Verify ComplianceResult aggregation (any block = blocked).

---

### Agent 2: Rules Engine

**Goal:** Build a pure evaluation engine as a new port + adapter.

**Files to create:**
- `src/ports/rules_engine.rs` — `RulesEngine` trait
- `src/adapters/rules_engine_impl.rs` — Implementation that loads rules from KV and evaluates
- `src/adapters/rule_loader.rs` — Parse TOML rule configs into Rule structs

**Files to modify:**
- `src/ports/mod.rs` — Add `pub mod rules_engine;`
- `src/adapters/mod.rs` — Add `pub mod rules_engine_impl; pub mod rule_loader;`

**RulesEngine port trait:**
```rust
pub trait RulesEngine {
    fn evaluate(&self, context: &FilingContext, rules: &[Rule]) -> ComplianceReport;
    fn select_rules(&self, jurisdiction: &str, trigger: &TriggerEvent, all_rules: &[Rule]) -> Vec<Rule>;
    fn resolve_priority(&self, matching_rules: Vec<Rule>) -> Vec<Rule>;
}
```

**Evaluation algorithm:**
1. Filter rules by `jurisdiction_id` matching context jurisdiction (plus "federal" for FRCP)
2. Filter by `trigger_event` matching context trigger
3. Check each rule's `condition` JSON against context fields
4. Sort by `RuleSource::priority()` (ascending = highest priority first)
5. For conflicts at same priority, more specific conditions win
6. Execute each rule's action, collect results
7. If ANY action is `Block` or `Enforce` and fails → `ComplianceReport.blocked = true`

**Condition evaluation — JSON matching:**
```rust
// Simple: {"case_type": ["civil", "criminal"]} → context.case_type in list
// Wildcard: {"case_type": ["*"]} → always matches
// Exclusion: {"document_type": ["motion"], "excluded_types": ["extension"]} → match with exclusions
// Boolean: {"and": [cond1, cond2]}, {"or": [cond1, cond2]}, {"not": cond}
```

**Loading rules from KV:**
- On first request, check if rules are loaded: `KV.get("rules_loaded:{jurisdiction}")`
- If not, load from embedded TOML (compiled into Wasm) or from KV admin upload
- Cache in a static/lazy structure for the request lifetime (Spin is request-scoped)

**Critical: Spin is request-scoped.** Each request gets a fresh Wasm instance. There is no persistent in-memory state between requests. Rules must be loaded from KV each request (or use Spin's component-level caching if available). Keep the rule set small enough that this is fast (ARWD is ~75 rules).

**Tests:**
- `test_evaluate_civil_complaint_passes_all_rules`
- `test_evaluate_unredacted_ssn_blocks_filing`
- `test_standing_order_overrides_local_rule`
- `test_condition_and_both_must_match`
- `test_condition_or_either_matches`
- `test_wildcard_matches_all_case_types`

---

### Agent 3: Deadline Engine (FRCP Rule 6)

**Goal:** Implement the complete FRCP Rule 6 day-counting algorithm.

**Files to create:**
- `src/ports/deadline_engine.rs` — `DeadlineEngine` trait
- `src/adapters/deadline_engine_impl.rs` — Full FRCP Rule 6 implementation

**Files to modify:**
- `src/ports/mod.rs` — Add `pub mod deadline_engine;`
- `src/adapters/mod.rs` — Add `pub mod deadline_engine_impl;`

**FRCP Rule 6 Algorithm (implement exactly):**
```
fn compute_deadline(trigger_date, period_days, service_method, jurisdiction) -> NaiveDate:
    1. Exclude the trigger date (start counting the NEXT day)
    2. Add service method days FIRST:
       - Mail, Leaving with clerk, Other Rule 5(b)(2)(C)-(F): +3 calendar days
       - Electronic service, personal delivery: +0
    3. Count the period:
       - If total period < 11 days: exclude weekends AND federal holidays while counting
       - If total period >= 11 days: count ALL calendar days (weekends and holidays included)
    4. If the resulting date falls on a weekend or federal holiday:
       extend to the NEXT business day
    5. Apply jurisdiction-specific adjustments:
       - ARWD: e-filing deadline is midnight CT (not midnight ET)
       - Judge-specific: "close of business" means office hours per standing order
```

**Federal holidays (hardcode 2024-2030, load extensions from KV):**
- New Year's Day, MLK Day, Presidents' Day, Memorial Day, Juneteenth
- Independence Day, Labor Day, Columbus Day, Veterans Day, Thanksgiving, Christmas
- Plus: court-specific closure dates from KV

**Holiday edge cases:**
- When holiday falls on Saturday → observed Friday
- When holiday falls on Sunday → observed Monday
- MLK: 3rd Monday of January (compute dynamically)
- Memorial Day: last Monday of May
- Thanksgiving: 4th Thursday of November

**Service method adjustment table:**
| Method | Additional Days |
|--------|----------------|
| Electronic | +0 |
| Personal delivery | +0 |
| Mail (USPS) | +3 calendar |
| Leaving with clerk | +3 calendar |
| Other Rule 5(b)(2)(C)-(F) | +3 calendar |

**Key ARWD deadlines to encode as test cases:**
| Event | Period | Rule |
|-------|--------|------|
| Motion filed → Response due | 14 days | LR 7.2(b) |
| Motion for summary judgment → Reply | 7 days | LR 7.2(c) |
| Service → Answer due | 21 days | FRCP 12(a) |
| Amended pleading → Response | 14 days | FRCP 15(a)(3) |
| Discovery request → Response | 30 days | FRCP 33/34 |

**Tests (comprehensive — this is where attorneys miss deadlines):**
- `test_14_day_response_normal_business_day` — filing Friday Jan 10 → due Fri Jan 24
- `test_14_day_response_lands_on_saturday` — extends to Monday
- `test_14_day_response_lands_on_sunday` — extends to Monday
- `test_14_day_response_lands_on_mlk_day` — extends to Tuesday
- `test_short_period_7_days_excludes_weekends` — only count business days
- `test_long_period_30_days_includes_weekends` — count all calendar days
- `test_mail_service_adds_3_then_counts` — 14 + 3 = 17 days, ≥11 so count calendar
- `test_electronic_service_no_addition` — 14 days, ≥11 count calendar
- `test_holiday_chain_christmas_newyears` — Dec 23 trigger with short period
- `test_memorial_day_computation` — last Monday of May, dynamically computed
- `test_backward_counting_pretrial_deadline` — "no later than 14 days before trial"

---

### Agent 4: Privacy Engine (FRCP 5.2)

**Goal:** Build PII detection and document restriction.

**Files to create:**
- `src/ports/privacy_engine.rs` — `PrivacyEngine` trait
- `src/adapters/privacy_engine_impl.rs` — Regex-based PII scanner

**Files to modify:**
- `src/ports/mod.rs` — Add `pub mod privacy_engine;`
- `src/adapters/mod.rs` — Add `pub mod privacy_engine_impl;`

**FRCP 5.2(a) — Required redactions:**

| PII Type | Detection Pattern | Required Format |
|----------|------------------|-----------------|
| SSN | `\b\d{3}[-\s]?\d{2}[-\s]?\d{4}\b` | Last 4 only: XXX-XX-1234 |
| Taxpayer ID | `\b\d{2}[-\s]?\d{7}\b` | Last 4 only |
| Date of Birth | `born|DOB|date of birth` + date pattern | Year only |
| Minor Name | Context-dependent (party metadata + NER) | Initials only |
| Financial Account | `\b\d{8,}\b` in financial context | Last 4 only |
| Home Address (criminal) | NER + criminal case type | City/state only |

**False positive filtering (critical):**
- Case numbers look like SSNs: `5:24-cv-05001` → NOT an SSN
- Phone numbers trigger account patterns: `(479) 555-1234` → NOT a financial account
- Dates in procedural context: "filed on 01/15/1990" → NOT a DOB unless preceded by "born"

**Auto-restricted document types (FRCP 5.2(b)) — block public access:**
- Unexecuted warrants/summons
- Presentence investigation (PSI) reports
- Statement of reasons (SOR)
- CJA financial affidavits
- Juvenile records
- Juror identifying info
- Sealed documents

**Scan pipeline:**
```
document_text → Run PII patterns (parallel) → Filter false positives
                                              → Check auto-restrict list
                                              → Generate PrivacyScanResult
If violations: BLOCK with specific PII locations and required format
If restricted doc type: Set access controls (counsel + staff only)
If clean: PASS
```

**Tests:**
- `test_unredacted_ssn_detected` — "John's SSN is 123-45-6789" → BLOCK
- `test_properly_redacted_ssn_passes` — "SSN: XXX-XX-6789" → PASS
- `test_case_number_not_false_positive` — "Case 5:24-cv-05001" → PASS (not an SSN)
- `test_phone_number_not_account` — "(479) 555-1234" → PASS
- `test_dob_with_born_keyword_detected` — "born January 15, 1990" → BLOCK
- `test_date_without_born_not_flagged` — "filed on January 15, 1990" → PASS
- `test_psi_report_auto_restricted` — PSI doc type → restricted access
- `test_multiple_pii_types_all_reported` — doc with SSN + DOB → both reported

---

### Agent 5: Filing Pipeline & Handlers

**Goal:** Wire everything together with new HTTP endpoints and integrate with existing handlers.

**Files to create:**
- `src/handlers/filing.rs` — Filing submission with full compliance pipeline
- `src/handlers/rules_admin.rs` — Rule CRUD (create/read/update/delete rules per jurisdiction)
- `src/adapters/spin_kv_rules_repository.rs` — Store/retrieve rules in Spin KV
- `config/jurisdictions/arwd/rules.toml` — ARWD rule pack (complete)
- `config/jurisdictions/arwd/court.toml` — Court config (divisions, judges, hours)
- `config/jurisdictions/arwd/fees.toml` — Fee schedule

**Files to modify:**
- `src/lib.rs` — Add routes for new handlers
- `src/handlers/mod.rs` — Add `pub mod filing; pub mod rules_admin;`
- `src/adapters/mod.rs` — Add `pub mod spin_kv_rules_repository;`

**Filing submission pipeline (POST /api/filings):**
```
1. Parse FilingSubmission from request body
2. Resolve jurisdiction from X-Court-District header
3. Load rules for jurisdiction from KV
4. Run PrivacyEngine::scan(document_text, case_type)
   → If violations: return 422 with PII locations
5. Run RulesEngine::evaluate(filing_context, rules)
   → If blocked: return 422 with rule citations
   → If warnings: include in response
6. Run DeadlineEngine::compute() for triggered deadlines
7. Create DocketEntry via existing docket repository
8. Generate NEF notification record
9. Return 201 with filing receipt + deadline chain + warnings
```

**Rules admin endpoints:**
```
GET    /api/rules?jurisdiction={id}           → List rules for jurisdiction
GET    /api/rules/:rule_id                    → Get specific rule
POST   /api/rules                             → Create rule (court admin)
PUT    /api/rules/:rule_id                    → Update rule
DELETE /api/rules/:rule_id                    → Soft-delete rule
POST   /api/rules/evaluate                    → Dry-run: test a filing against rules
GET    /api/rules/jurisdictions               → List configured jurisdictions
POST   /api/rules/jurisdictions/:id/load      → Load rule pack from TOML config
```

**ARWD rules.toml format (Agent 5 creates, Agent 2 parses):**
```toml
[[rules]]
rule_id = "arwd-lr-7.2b"
jurisdiction_id = "arwd"
rule_source = "LocalRule"
rule_number = "7.2(b)"
rule_category = "Deadline"
trigger_event = "MotionFiled"
action = "Calculate"
source_text = "A response to a motion must be filed within fourteen (14) days..."

[rules.condition]
document_type = ["motion"]
excluded_types = ["summary_judgment"]

[rules.parameters]
deadline_days = 14
deadline_description = "Response to motion due"
```

**ARWD court.toml:**
```toml
[court]
name = "Western District of Arkansas"
code = "arwd"
timezone = "America/Chicago"
filing_deadline = "midnight"

[[divisions]]
name = "Fort Smith"
code = "FSM"
staffed = true

[[divisions]]
name = "Fayetteville"
code = "FAY"
staffed = true

[[divisions]]
name = "Harrison"
code = "HAR"
staffed = true
criminal_routing = "FAY"  # Criminal cases route to Fayetteville

[[divisions]]
name = "Hot Springs"
code = "HSP"
staffed = false
routing = "FSM"  # All cases route to Fort Smith

[[divisions]]
name = "El Dorado"
code = "ELD"
staffed = false
routing = "FSM"

[[divisions]]
name = "Texarkana"
code = "TXK"
staffed = false
routing = "FSM"

[[judges]]
name = "Timothy L. Brooks"
initials = "TLB"
type = "District"
division = "FAY"

[[judges]]
name = "P.K. Holmes III"
initials = "PKH"
type = "District"
division = "FSM"

[[judges]]
name = "Susan O. Hickey"
initials = "SOH"
type = "District"
division = "FSM"

[[judges]]
name = "Timothy L. Barnes"
initials = "TLB2"
type = "Magistrate"
division = "FAY"
```

**Integration with existing handlers (minimal, non-breaking):**

In `src/handlers/criminal_case.rs` — add an OPTIONAL compliance check on case creation. Do NOT break existing behavior. Use a feature flag or query param (`?compliance=true`) to opt in:
```rust
// Only if compliance mode enabled
if compliance_enabled {
    let rules = load_rules_for_jurisdiction(&jurisdiction);
    let report = rules_engine.evaluate(&filing_context, &rules);
    if report.blocked {
        return compliance_error_response(report);
    }
}
// ... existing case creation logic unchanged ...
```

---

## Coordination Rules

1. **No agent modifies another agent's primary files** — use `mod.rs` exports as the integration boundary
2. **All agents depend on Agent 1's types** — Agent 1 should complete domain types first, or all agents work against the type signatures defined above
3. **Agents 2, 3, 4 are independent** — rules engine, deadline engine, and privacy engine have no mutual dependencies
4. **Agent 5 depends on Agents 1-4** — filing pipeline wires everything together
5. **All agents follow existing Verdictum code patterns:**
   - Use `serde` for serialization
   - Use `spin_sdk::key_value::Store` for persistence
   - Return JSON responses via `spin_sdk::http::Response`
   - Match existing error handling patterns in `src/error.rs`
6. **Test within Spin constraints** — `#[cfg(test)]` module tests don't need Spin SDK. Integration tests that touch KV require `spin test` framework or mock the Store trait.

## Execution Order

```
Phase 1 (parallel):
  Agent 1: Domain types  ──┐
  Agent 2: Rules engine  ──┤  (can work against type signatures)
  Agent 3: Deadline engine ┤
  Agent 4: Privacy engine ─┘

Phase 2 (after Phase 1 compiles):
  Agent 5: Filing pipeline + handlers + ARWD config + wiring
```

## Verification Checklist

After all agents complete:

- [ ] `cargo build --target wasm32-wasip1` compiles with zero errors
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo test` passes all unit tests
- [ ] All existing 350+ endpoints still respond (no regressions)
- [ ] `POST /api/rules/evaluate` with a mock filing returns correct compliance result
- [ ] `POST /api/filings` with an unredacted SSN returns 422 with PII location
- [ ] `POST /api/filings` with a civil complaint in ARWD returns correct deadline chain
- [ ] `GET /api/rules?jurisdiction=arwd` returns all 75 ARWD rules
- [ ] Deadline computation for "14 day response, mail service, lands on MLK Day" is correct
- [ ] Harrison Division criminal case routes to Fayetteville

## ARWD Rule Pack Reference

**Key rules to encode (Agent 5 writes TOML, all agents test against):**

| Rule | Source | Trigger | Action | Summary |
|------|--------|---------|--------|---------|
| LR 5.1(b) | LocalRule | DocumentUploaded | Enforce | Electronic signatures must include /s/ format with bar number |
| LR 5.5(e) | LocalRule | AmendedPleadingFiled | Calculate | 7-day deadline to file amended pleading |
| LR 7.2(b) | LocalRule | MotionFiled | Calculate | 14-day response deadline |
| LR 7.2(c) | LocalRule | SummaryJudgmentMotionFiled | Calculate | 7-day reply deadline |
| LR 40.1 | LocalRule | CaseOpened | Validate | Random judge assignment required |
| LR 54.1 | LocalRule | JudgmentEntered | Calculate | 14-day attorney fees motion deadline |
| LR 56.1 | LocalRule | SummaryJudgmentFiled | Enforce | Statement of facts required |
| LR 77.1 | LocalRule | CriminalCaseOpenedHarrison | Enforce | Route to Fayetteville |
| Admin III.A.1 | AdminProcedure | CaseInitiatingDocFiled | Enforce | Must be conventional (not electronic) |
| Admin III.A.3 | AdminProcedure | DocumentFiled | Validate | Midnight CT filing deadline |
| Admin III.E | AdminProcedure | ProposedOrderFiled | Enforce | Must be Word format, not PDF |
| FRCP 5.2 | FRCP | DocumentUploaded | Enforce | PII must be redacted |
| FRCP 6 | FRCP | DeadlineTriggered | Calculate | Full day-counting algorithm |
| FRCP 12(a) | FRCP | ServiceComplete | Calculate | 21-day answer deadline |
| FRCP 15(a)(3) | FRCP | AmendedPleadingServed | Calculate | 14-day response deadline |
| FRCP 33/34 | FRCP | DiscoveryRequestServed | Calculate | 30-day response deadline |

---

## Quick Reference: Type Signatures

All agents should code against these signatures. Agent 1 implements them; others import them.

```rust
// src/domain/rule.rs
pub enum RuleSource { StandingOrder, LocalRule, AdminProcedure, FRCP, Statute }
pub enum RuleAction { Enforce, Warn, Validate, Calculate, Block, Notify }
pub enum RuleCategory { Filing, Deadline, Privacy, Fees, Service, JudgeAssignment, Format }
pub enum TriggerEvent { MotionFiled, ComplaintFiled, DocumentUploaded, CaseOpened, ServiceComplete, DeadlineTriggered, JudgmentEntered, AmendedPleadingFiled, ProposedOrderFiled, CaseInitiatingDocFiled, ... }

pub struct Rule { /* see Agent 1 section */ }
pub struct ComplianceReport { pub results: Vec<RuleResult>, pub blocked: bool, pub block_reasons: Vec<String>, pub warnings: Vec<String>, pub deadlines: Vec<DeadlineResult> }
pub struct RuleResult { pub rule_id: String, pub matched: bool, pub action_taken: RuleAction, pub message: String }

// src/domain/deadline_calc.rs
pub enum ServiceMethod { Electronic, PersonalDelivery, Mail, LeavingWithClerk, Other }
pub struct DeadlineResult { pub due_date: String, pub description: String, pub rule_citation: String, pub computation_notes: String }
pub struct FederalHoliday { pub date: String, pub name: String }

// src/domain/privacy.rs
pub enum PiiType { Ssn, TaxpayerId, DateOfBirth, MinorName, FinancialAccount, HomeAddress }
pub struct PiiMatch { pub pii_type: PiiType, pub location: (usize, usize), pub original_text: String, pub required_format: String }
pub struct PrivacyScanResult { pub clean: bool, pub violations: Vec<PiiMatch>, pub restricted: bool, pub restriction_reason: Option<String> }

// src/domain/filing_pipeline.rs
pub struct FilingContext { pub case_type: String, pub document_type: String, pub filer_role: String, pub jurisdiction_id: String, pub division: Option<String>, pub assigned_judge: Option<String>, pub service_method: Option<ServiceMethod>, pub metadata: serde_json::Value }

// src/domain/nef.rs
pub struct NoticeOfElectronicFiling { pub filing_id: String, pub case_number: String, pub filed_at: String, pub document_type: String, pub filer_name: String, pub docket_text: String, pub recipients: Vec<NefRecipient> }
pub struct NefRecipient { pub name: String, pub email: Option<String>, pub delivery_method: DeliveryMethod }
pub enum DeliveryMethod { ElectronicNef, PaperCopy }
```
