# Lexodus - Federal Judicial Case Management System

A comprehensive federal court case management system built with [Spin](https://github.com/fermyon/spin) and Rust, compiled to WebAssembly. **Lexodus** is a complete judicial case management platform for federal district courts, featuring modern **Hexagonal Architecture** patterns and demonstrating how complex judicial workflows can be managed efficiently in a cloud-native environment.

## Features

### Core Platform Features
- 🚀 **Fast and Lightweight** - Built with Rust and compiled to WebAssembly
- 💾 **Persistent Storage** - Uses Spin's built-in key-value store with atomic operations
- 📚 **OpenAPI Documentation** - Interactive API documentation with comprehensive Swagger UI
- ❤️ **Health Check** - Advanced monitoring endpoints for deployment and health checks
- 🛡️ **Error Handling** - Comprehensive error handling with typed errors and consistent JSON responses
- 🏗️ **Hexagonal Architecture** - Clean separation between domain logic and infrastructure
- 🔌 **Multi-Tenant Support** - Enterprise-grade multi-tenancy with data isolation
- 🎛️ **Feature Flag Management** - Dynamic feature toggling and A/B testing capabilities

### 1. Federal Case Management System (100+ Endpoints)
**Complete Lexodus Implementation for Federal District Courts**
- ⚖️ **Criminal Case Lifecycle** - Full case management from filing through sentencing
- 📋 **Plea Management** - Track defendant pleas (Guilty, Not Guilty, Nolo Contendere, Alford)
- 📅 **Court Events** - Schedule arraignments, hearings, trials, and sentencing
- 📜 **Motion System** - File, track, and rule on pretrial and post-trial motions
- 🎯 **Federal Case Status** - Track through all federal court stages
- 📊 **Real-Time Statistics** - Comprehensive case metrics and performance analytics
- 🔍 **Advanced Search** - Multi-criteria case search and filtering
- 👥 **Defendant Management** - Track multiple defendants per case
- 🗂️ **Evidence Tracking** - Comprehensive evidence management and chain of custody

### 2. Judge Management System (26 Endpoints)
**Complete Federal Judge Administration**
- 👨‍⚖️ **Judge Profiles** - Comprehensive judge information and credentials
- 📊 **Workload Management** - Automated case load balancing and distribution
- 🔄 **Assignment Engine** - Intelligent case assignment with conflict checking
- 🚫 **Recusal System** - Complete recusal filing, review, and approval workflow
- 🛡️ **Conflict of Interest** - Advanced conflict detection and management
- 📅 **Availability Tracking** - Judge schedule and vacation management
- 🏛️ **District Assignment** - Multi-district judge management
- 📈 **Performance Metrics** - Judge productivity and case completion statistics

### 3. Docket & Calendar Management (28 Endpoints)
**Professional Court Scheduling and Documentation**
- 📋 **Electronic Docket** - Complete docket entry management with attachments
- 📅 **Court Calendar** - Advanced scheduling with conflict detection
- 🏛️ **Courtroom Management** - Resource allocation and utilization tracking
- 📊 **Utilization Analytics** - Courtroom and judge schedule optimization
- 🔍 **Docket Search** - Advanced search across all court filings
- 📄 **Docket Sheet Generation** - Professional docket sheet formatting
- 🔒 **Sealed Documents** - Secure handling of confidential filings
- 📈 **Filing Statistics** - Comprehensive filing analytics and trends

### 4. Deadline Tracking & Compliance (26 Endpoints)
**Federal Rules Compliance and Deadline Management**
- ⏰ **FRCP Deadline Calculator** - Automated Federal Rules of Civil Procedure calculations
- 📅 **Deadline Tracking** - Comprehensive deadline management with escalation
- 🚨 **Compliance Monitoring** - Real-time compliance tracking and reporting
- 📧 **Automated Reminders** - Multi-channel reminder system with acknowledgment
- 📊 **Performance Metrics** - Deadline compliance statistics and trends
- 🏛️ **Jurisdictional Deadlines** - Critical deadline tracking with special handling
- 📋 **Extension Management** - Complete extension request and approval workflow
- 📈 **Risk Analytics** - Deadline risk assessment and early warning systems

### 5. Speedy Trial Act Compliance
**Federal Speedy Trial Act Implementation**
- ⏱️ **70-Day Clock** - Automated Speedy Trial Act timeline tracking
- 📅 **Excludable Delays** - Comprehensive delay categorization and tracking
- 🚨 **Violation Detection** - Automatic detection of Speedy Trial violations
- 📊 **Compliance Reporting** - Federal reporting requirements compliance
- 🎯 **Deadline Management** - Critical deadline tracking with escalation
- 📋 **Delay Justification** - Complete documentation of excludable time periods

### 6. Feature Flag Management (10 Endpoints)
**Enterprise Feature Management**
- 🎛️ **Dynamic Toggles** - Real-time feature enabling/disabling
- 🔧 **Implementation Tracking** - Feature development lifecycle management
- 🚦 **Deployment Gates** - Controlled feature rollouts
- 📊 **Usage Analytics** - Feature adoption and usage metrics
- 🎯 **A/B Testing** - Experimental feature testing framework
- 🔒 **Access Control** - Role-based feature access management

### 7. Judicial Orders System (18 Endpoints)
**Complete Court Order Management**
- 📜 **Order Creation** - Comprehensive order drafting and management
- ✍️ **Electronic Signatures** - Secure electronic signature workflow
- 📋 **Order Templates** - Reusable templates with variable substitution
- 📮 **Service Tracking** - Complete service of process management
- ⏰ **Expiration Tracking** - Automatic expiration date monitoring
- 🔒 **Sealed Orders** - Secure handling of sealed court orders
- 📊 **Order Statistics** - Comprehensive analytics and reporting
- 📄 **Template Management** - Dynamic order template system

### 8. Judicial Opinions System (20 Endpoints)
**Federal Court Opinion Management**
- 📝 **Opinion Drafting** - Complete opinion lifecycle from draft to publication
- 📚 **Version Control** - Draft versioning with collaboration features
- ⚖️ **Precedential Tracking** - Binding vs non-binding opinion management
- 📖 **Citation Management** - Legal citation tracking with treatment analysis
- 🏛️ **Headnotes** - Professional headnote management system
- 👨‍⚖️ **Vote Tracking** - Judge voting and concurrence/dissent management
- 📊 **Citation Analytics** - Citation frequency and treatment statistics
- 🔍 **Opinion Search** - Full-text search across all opinions

### 9. Attorney Management System (67 Endpoints)
**Federal Attorney and Legal Representation Administration**
- ⚖️ **Attorney Profiles** - Comprehensive attorney bar admissions and credentials
- 📋 **Bar Admissions** - Multi-jurisdiction bar status tracking
- 🏛️ **Federal Court Admissions** - ECF registration and federal court practice
- 💼 **CJA Panel Management** - Criminal Justice Act panel membership and appointments
- 🔍 **Conflict Checking** - Advanced conflict of interest detection
- 📊 **Performance Metrics** - Win rates, case duration, and practice analytics
- 🌐 **Multi-Language Support** - Attorney language capabilities for diverse clients
- 👥 **Pro Hac Vice** - Temporary admission tracking for out-of-state counsel

### 10. Sentencing Management System (28 Endpoints)
**Federal Criminal Sentencing Guidelines**
- ⚖️ **Guidelines Calculation** - Automated federal sentencing guidelines computation
- 📈 **Enhancement Tracking** - Offense level enhancements and adjustments
- 📉 **Reduction Management** - Acceptance of responsibility and cooperation reductions
- 📊 **Criminal History** - Prior conviction scoring and category determination
- 🎯 **Departure Tracking** - Upward and downward departure documentation
- 📋 **Variance Management** - Non-guidelines sentence justification
- 🔒 **Mandatory Minimums** - Statutory minimum sentence tracking
- 📄 **Sentencing Memoranda** - Prosecution and defense sentencing position management

### 11. Party Management System (24 Endpoints)
**Case Participant and Party Administration**
- 👥 **Party Profiles** - Comprehensive participant information management
- 🏛️ **Representation Tracking** - Attorney-client relationship management
- 📋 **Contact Management** - Multi-channel contact information
- 🔍 **Role Assignment** - Plaintiff, defendant, witness, and third-party roles
- 📊 **Party Analytics** - Participation history and case involvement
- 🌐 **Corporate Entities** - Business and organizational party management
- 📄 **Service Management** - Service of process tracking
- 🔒 **Protected Parties** - Sealed and confidential party information

### 12. Multi-Tenant Administration (94 Federal Districts)
**Enterprise Multi-Tenancy with Physical Data Isolation**
- 🏛️ **Federal District Isolation** - Complete data separation for all 94 federal judicial districts
- 📂 **Physical File Separation** - Each district gets its own database file for complete isolation
- 🔒 **Tenant-Aware Routing** - Automatic tenant detection via headers, subdomains, or query params
- 📊 **Per-District Analytics** - Usage statistics and resource monitoring per district
- 🎯 **District-Specific Configuration** - Custom settings and features per federal district

### 13. PDF Document Generation System (18 Endpoints)
**Federal Court Forms and Document Generation with Supreme Court Formatting**
- 📄 **Rule 16(b) Orders** - Auto-generate scheduling orders for criminal cases
- ✍️ **Electronic Signatures** - Apply judge signatures with secure authentication and bordered signature boxes
- 📋 **Federal Forms Library** - Complete implementation of official federal court forms:
  - **Form AO 455** - Waiver of Indictment
  - **Form AO 199A** - Order Setting Conditions of Release
  - **Form AO 245B** - Judgment in a Criminal Case
- 🏛️ **Court Orders** - Generate custom court orders with electronic signing
- 📝 **Minute Entries** - Professional minute entry generation with clerk signatures
- 🔐 **Signature Management** - Secure storage and application of judicial signatures with SHA256 hashing
- 📊 **Document Templates** - Reusable templates with variable substitution
- 🎯 **Auto-Generation** - Test endpoints with pre-populated data for all forms
- 📁 **Batch Generation** - Generate multiple documents in a single API call with `/api/pdf/batch`
- 📦 **Dual Response Formats** - Support for both application/pdf and application/json (base64) based on Accept header
- ⚖️ **Supreme Court Rule 33 Formatting** - Professional legal document formatting:
  - 8.5x11" page size (Letter)
  - Times-Roman font
  - 1-inch margins
  - Double-spacing
  - Proper text wrapping
  - Centered headers and content
- 🏗️ **Hexagonal Architecture** - Clean separation between PDF generation ports and adapters

### 14. Legacy ToDo System (5 Endpoints)
**Simple Task Management (Demo/Testing)**
- 📝 **CRUD Operations** - Basic task management functionality
- 🔄 **Status Toggle** - Task completion tracking
- 📄 **Pagination** - Efficient large list handling
- 🗑️ **Soft Delete** - Recoverable deletion functionality

## API Documentation

### Interactive Documentation

Once the application is running, you can access the comprehensive interactive Swagger UI documentation at:
- **Local Development**: `http://localhost:3000/docs`
- **OpenAPI JSON Specification**: `http://localhost:3000/docs/openapi-description.json`

The interactive documentation includes complete schemas, examples, and testing capabilities for all 250+ endpoints across the 14 major system modules.

### System Overview

The Lexodus system provides **350+ REST API endpoints** organized into these major modules:

| Module | Endpoints | Description |
|--------|-----------|-------------|
| **Attorney Management** | 67 | Attorney profiles, admissions, conflicts, and CJA panel management |
| **Sentencing Management** | 28 | Federal sentencing guidelines calculation and tracking |
| **Judge Management** | 26 | Judge administration, assignment, and conflict tracking |
| **Docket & Calendar** | 28 | Court scheduling and document management |
| **Deadline Tracking** | 26 | Federal rules compliance and deadline management |
| **Party Management** | 24 | Case participant and party administration |
| **Case Management** | ~20 | Core criminal case lifecycle management |
| **Judicial Orders** | 18 | Court order creation, signing, and service tracking |
| **Judicial Opinions** | 20 | Opinion drafting, publication, and citation management |
| **Feature Flags** | 10 | Enterprise feature management |
| **Speedy Trial Act** | 8 | Federal Speedy Trial Act compliance |
| **Multi-Tenant Admin** | 94 districts | Federal district data isolation and management |
| **Legacy ToDo** | 5 | Simple task management (demo/testing) |
| **Health & Docs** | 3 | System health and documentation |

**Total: 350+ Core Endpoints** across 14 major system modules with complete federal court management capabilities

## Multi-Tenant Architecture with Physical Data Isolation

The system implements enterprise-grade multi-tenancy for all 94 federal judicial districts with complete data isolation:

### Tenant Identification
Tenant detection occurs automatically through multiple methods:
- **HTTP Headers**: `X-Tenant-ID` or `X-Court-District` headers
- **Subdomain Routing**: `sdny.lexodus.gov` routes to SDNY district
- **Query Parameters**: `?tenant=sdny` for explicit tenant selection
- **Default Fallback**: Uses "default" tenant if none specified

### Physical File Separation
Each federal district gets its own isolated database file:
```toml
# runtime-config.toml
[key_value_store.sdny]
type = "spin"
path = ".spin/stores/sdny.db"

[key_value_store.edny]
type = "spin"
path = ".spin/stores/edny.db"
# ... configured for all 94 federal districts
```

### Repository Factory Pattern
Centralized repository creation with automatic tenant routing:
```rust
use crate::utils::repository_factory::RepositoryFactory;

// In any handler:
let attorney_repo = RepositoryFactory::attorney_repo(&req);
let case_repo = RepositoryFactory::case_repo(&req);
// Automatically uses the correct tenant's isolated database
```

### Development vs Production
- **Development**: Uses dev-config.toml with `./dev.sh` for rapid development with sample districts
- **Production**: Uses runtime-config.toml with physical separation via `./prod.sh`

## Hexagonal Architecture Demonstration

The Criminal Case Management API demonstrates **Hexagonal Architecture** (Ports and Adapters):

```
┌─────────────────────────────────────────────┐
│              HTTP Handlers                  │  ← Inbound Adapters
│         (handlers/criminal_case.rs)         │
├─────────────────────────────────────────────┤
│            Repository Port                  │  ← Port (Interface)
│        (ports/case_repository.rs)           │
├─────────────────────────────────────────────┤
│            Domain Model                     │  ← Core Business Logic
│        (domain/criminal_case.rs)            │
├─────────────────────────────────────────────┤
│         Spin KV Repository                  │  ← Outbound Adapter
│   (adapters/spin_kv_case_repository.rs)     │
└─────────────────────────────────────────────┘
```

**Benefits:**
- **Testability**: Handlers use repository interfaces, making them easy to test with mocks
- **Flexibility**: Storage can be swapped (Spin KV → PostgreSQL) without changing domain logic
- **Clean Separation**: Business logic has no infrastructure dependencies

---

## 🏛️ Federal Case Management API Endpoints

### Key Federal Case Management Endpoints

#### Create Criminal Case
```http
POST /api/cases
Content-Type: application/json

{
  "title": "United States v. Smith",
  "description": "Bank fraud and money laundering charges",
  "crimeType": "white_collar_crime",
  "assignedJudge": "Judge Johnson",
  "location": "Northern District of California"
}
```

**Response:** `201 Created`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "caseNumber": "2024-CR-00123",
  "title": "United States v. Smith",
  "status": "filed",
  "priority": "medium",
  "openedAt": "2024-01-15T10:30:00Z",
  "assignedJudge": "Judge Johnson"
}
```

#### Enter Defendant Plea
```http
POST /api/cases/:id/plea
Content-Type: application/json

{
  "defendant": "John Smith",
  "pleaType": "not_guilty",
  "pleaDate": "2024-01-20T14:30:00Z",
  "counts": ["bank_fraud", "money_laundering"]
}
```

#### Schedule Court Event
```http
POST /api/cases/:id/events
Content-Type: application/json

{
  "eventType": "arraignment",
  "scheduledDate": "2024-02-15T09:00:00Z",
  "courtroom": "A-101",
  "estimatedDuration": 60,
  "participants": ["Judge Johnson", "John Smith", "Public Defender"]
}
```

#### File Motion
```http
POST /api/cases/:id/motions
Content-Type: application/json

{
  "motionType": "suppress_evidence",
  "title": "Motion to Suppress Evidence",
  "filedBy": "Defense Attorney",
  "dueDate": "2024-03-01T17:00:00Z",
  "grounds": "Fourth Amendment violation"
}
```

---

## 👨‍⚖️ Judge Management API Endpoints

### Core Judge Management Operations

#### Create Judge Profile
```http
POST /api/judges
Content-Type: application/json

{
  "name": "Hon. Patricia Johnson",
  "title": "District Judge",
  "district": "Northern District of California",
  "appointedDate": "2018-05-15",
  "status": "active",
  "maxCaseload": 150
}
```

#### Assign Case to Judge
```http
POST /api/assignments
Content-Type: application/json

{
  "caseId": "550e8400-e29b-41d4-a716-446655440000",
  "judgeId": "judge-123",
  "assignmentType": "random",
  "assignedDate": "2024-01-15T10:30:00Z"
}
```

#### Check for Conflicts of Interest
```http
GET /api/judges/conflicts/check/:party
```

**Response:** `200 OK`
```json
{
  "hasConflicts": true,
  "conflictedJudges": [
    {
      "judgeId": "judge-456",
      "judgeName": "Judge Wilson",
      "conflictType": "financial_interest",
      "description": "Previous employment at defendant's company"
    }
  ]
}
```

#### File Recusal Motion
```http
POST /api/judges/:judge_id/recusals
Content-Type: application/json

{
  "caseId": "550e8400-e29b-41d4-a716-446655440000",
  "reason": "prior_relationship",
  "description": "Former law school classmate of defendant",
  "filedBy": "Hon. Patricia Johnson"
}
```

#### Get Judge Workload Statistics
```http
GET /api/judges/workload
```

**Response:** `200 OK`
```json
{
  "averageCaseload": 127,
  "judges": [
    {
      "judgeId": "judge-123",
      "name": "Judge Johnson",
      "activeCases": 142,
      "pendingCases": 38,
      "utilization": 94.7
    }
  ]
}
```

---

## 📋 Docket & Calendar Management API Endpoints

### Electronic Filing and Scheduling

#### Create Docket Entry
```http
POST /api/docket/entries
Content-Type: application/json

{
  "caseId": "550e8400-e29b-41d4-a716-446655440000",
  "entryType": "motion",
  "title": "Motion to Suppress Evidence",
  "filedBy": "Defense Attorney",
  "documentCount": 3,
  "isSealed": false,
  "serviceRequired": true
}
```

#### Schedule Court Event
```http
POST /api/calendar/events
Content-Type: application/json

{
  "caseId": "550e8400-e29b-41d4-a716-446655440000",
  "eventType": "hearing",
  "title": "Motion to Suppress Hearing",
  "scheduledDateTime": "2024-03-15T14:00:00Z",
  "courtroom": "A-205",
  "judgeId": "judge-123",
  "estimatedDuration": 120
}
```

#### Get Docket Sheet
```http
GET /api/docket/sheet/:case_id
```

**Response:** `200 OK`
```json
{
  "caseNumber": "2024-CR-00123",
  "title": "United States v. Smith",
  "judge": "Hon. Patricia Johnson",
  "entries": [
    {
      "entryNumber": 1,
      "date": "2024-01-15T10:30:00Z",
      "description": "Criminal Complaint",
      "filedBy": "AUSA Thompson"
    }
  ]
}
```

#### Find Available Court Time
```http
GET /api/calendar/available-slot/:judge_id?duration=60&preferredDate=2024-03-01
```

**Response:** `200 OK`
```json
{
  "availableSlots": [
    {
      "startTime": "2024-03-01T14:00:00Z",
      "endTime": "2024-03-01T15:00:00Z",
      "courtroom": "A-205"
    }
  ]
}
```

---

## ⏰ Deadline Tracking & Compliance API Endpoints

### Federal Rules Compliance Management

#### Calculate FRCP Deadlines
```http
POST /api/deadlines/calculate
Content-Type: application/json

{
  "triggerEvent": "service_of_process",
  "triggerDate": "2024-01-15",
  "ruleType": "FRCP",
  "specificRule": "12(b)"
}
```

**Response:** `200 OK`
```json
{
  "deadlines": [
    {
      "description": "Answer or Motion to Dismiss",
      "dueDate": "2024-02-15",
      "rule": "FRCP 12(a)(1)(A)",
      "isJurisdictional": false,
      "daysCalculated": 21
    }
  ]
}
```

#### Create Critical Deadline
```http
POST /api/deadlines
Content-Type: application/json

{
  "caseId": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Discovery Cutoff",
  "description": "All discovery must be completed",
  "dueDate": "2024-06-01T17:00:00Z",
  "deadlineType": "discovery",
  "isJurisdictional": false,
  "reminderDays": [30, 14, 7, 1]
}
```

#### Request Extension
```http
POST /api/deadlines/:deadline_id/extensions
Content-Type: application/json

{
  "requestedDate": "2024-06-15T17:00:00Z",
  "reason": "Complex expert testimony preparation required",
  "requestedBy": "Defense Attorney",
  "supportingFacts": "Case involves highly technical financial instruments"
}
```

#### Get Compliance Statistics
```http
GET /api/compliance/stats
```

**Response:** `200 OK`
```json
{
  "totalDeadlines": 1250,
  "metOnTime": 1127,
  "extensions": 98,
  "missed": 25,
  "complianceRate": 90.2,
  "avgExtensionDays": 14
}
```

---

## ⏱️ Speedy Trial Act Compliance API Endpoints

### Federal Speedy Trial Act Management

#### Initialize Speedy Trial Clock
```http
POST /api/speedy-trial/:case_id
Content-Type: application/json

{
  "indictmentDate": "2024-01-15",
  "defendantInCustody": true,
  "trialDate": "2024-03-15"
}
```

#### Add Excludable Delay
```http
POST /api/speedy-trial/:case_id/delays
Content-Type: application/json

{
  "delayType": "pretrial_motions",
  "startDate": "2024-02-01",
  "endDate": "2024-02-15",
  "description": "Motion to suppress evidence hearing and ruling",
  "statutoryBasis": "18 USC 3161(h)(1)(D)"
}
```

#### Check Approaching Deadlines
```http
GET /api/speedy-trial/approaching
```

**Response:** `200 OK`
```json
{
  "approachingCases": [
    {
      "caseId": "550e8400-e29b-41d4-a716-446655440000",
      "caseNumber": "2024-CR-00123",
      "daysRemaining": 12,
      "trialDate": "2024-03-15",
      "riskLevel": "high"
    }
  ]
}
```

---

## 🎛️ Feature Flag Management API Endpoints

### Enterprise Feature Control

#### Get All Features
```http
GET /api/features
```

**Response:** `200 OK`
```json
{
  "features": [
    {
      "path": "case_management.advanced_search",
      "enabled": true,
      "implementation": "complete",
      "rolloutPercentage": 100
    },
    {
      "path": "calendar.auto_scheduling",
      "enabled": false,
      "implementation": "in_progress",
      "rolloutPercentage": 0
    }
  ]
}
```

#### Toggle Feature
```http
PATCH /api/features
Content-Type: application/json

{
  "path": "deadline_tracking.ai_predictions",
  "enabled": true
}
```

---

## 📄 PDF Document Generation API Endpoints

### Generate Court Documents with Supreme Court Rule 33 Formatting

#### Generate Rule 16(b) Scheduling Order - PDF Format
```http
POST /api/pdf/rule16b/pdf
X-Tenant-Id: sdny
Content-Type: application/json

{
  "case_number": "24-CR-00123",
  "defendant_names": "John Doe, Jane Smith",
  "judge_name": "Hon. Patricia Johnson",
  "trial_date": "2024-06-15",
  "discovery_deadline": "2024-04-01",
  "motion_deadline": "2024-05-01",
  "pretrial_conference_date": "2024-06-01",
  "judge_id": "123e4567-e89b-12d3-a456-426614174000"
}
```
**Response:** Binary PDF file suitable for direct download

#### Generate Rule 16(b) Order - JSON Format with Base64 PDF
```http
POST /api/pdf/rule16b/json
X-Tenant-Id: sdny
Content-Type: application/json

{
  "case_number": "24-CR-00123",
  "defendant_names": "John Doe",
  "judge_name": "Hon. Patricia Johnson"
}
```
**Response:** `200 OK`
```json
{
  "pdf_base64": "JVBERi0xLjQKJeLj...",
  "filename": "rule16b_order_24-CR-00123.pdf",
  "content_type": "application/pdf"
}
```

#### Store Judge Electronic Signature
```http
POST /api/signatures
X-Tenant-Id: sdny
Content-Type: application/json

{
  "judge_id": "123e4567-e89b-12d3-a456-426614174000",
  "signature_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgA..."
}
```

#### Retrieve Judge Signature (Tenant-Isolated)
```http
GET /api/signatures/123e4567-e89b-12d3-a456-426614174000
X-Tenant-Id: sdny
```
**Response:** `200 OK`
```json
{
  "judge_id": "123e4567-e89b-12d3-a456-426614174000",
  "signature_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgA...",
  "uploaded_at": "2025-09-18T06:22:21.867663+00:00",
  "signature_hash": "624fc0a006e31b1610bb226be8b1a87911b9c54a390e789e1f30c2517d64db19"
}
```

#### Batch Generate Multiple Documents
```http
POST /api/pdf/batch
X-Tenant-Id: sdny
Content-Type: application/json

{
  "requests": [
    {
      "type": "rule16b",
      "case_number": "24-CR-00123",
      "defendant_names": "John Doe"
    },
    {
      "type": "waiver_of_indictment",
      "case_number": "24-CR-00456",
      "defendant_name": "Jane Smith"
    }
  ]
}
```

---

## 🏢 Multi-Tenant Administration API Endpoints

### Enterprise Tenant Management

#### Initialize New Tenant
```http
POST /api/admin/init-tenant
Content-Type: application/json

{
  "tenantId": "district_ndca",
  "name": "Northern District of California",
  "adminEmail": "admin@ndca.uscourts.gov",
  "features": ["case_management", "judge_management", "docket_management"]
}
```

#### Get Tenant Statistics
```http
GET /api/admin/tenant-stats
```

**Response:** `200 OK`
```json
{
  "tenants": [
    {
      "tenantId": "district_ndca",
      "activeCases": 3250,
      "activeJudges": 28,
      "storageUsed": "2.3GB",
      "apiCalls": 125000
    }
  ]
}
```

---

## 📝 Legacy ToDo System API Endpoints

### Simple Task Management (Demo/Testing)

#### Get All ToDo Items (with Pagination)
```http
GET /api/todos?page=1&limit=20&completed=false
```
Returns a paginated list of active (non-deleted) ToDo items.

**Query Parameters:**
- `page` (optional): Page number (default: 1, min: 1)
- `limit` (optional): Items per page (default: 20, min: 1, max: 100)
- `completed` (optional): Filter by completion status (true/false)

**Response:** `200 OK`
```json
{
  "items": [
    {
      "id": "059c7906-ce72-4433-94df-441beb14d96a",
      "contents": "Buy groceries",
      "isCompleted": false
    }
  ],
  "total": 42,
  "page": 1,
  "limit": 20,
  "totalPages": 3,
  "hasNext": true,
  "hasPrevious": false
}
```

#### Health Check
```http
GET /api/health
```
Check the health status of the API and storage connectivity.

**Response:** `200 OK` | `503 Service Unavailable`
```json
{
  "status": "healthy",
  "version": "4.0.0",
  "storage": "connected",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Get ToDo Item by ID
```http
GET /api/todos/:id
```
Retrieve a specific ToDo item using its UUID.

**Response:** `200 OK` | `404 Not Found` | `400 Bad Request`
```json
{
  "id": "059c7906-ce72-4433-94df-441beb14d96a",
  "contents": "Buy groceries",
  "isCompleted": false
}
```

#### Create ToDo Item
```http
POST /api/todos
Content-Type: application/json

{
  "contents": "Buy groceries"
}
```
Creates a new ToDo item with the provided contents.

**Validation:**
- Content must not be empty
- Content must not exceed 1000 characters

**Response:** `201 Created`
- Headers: `Location: /api/todos/{id}`
```json
{
  "id": "059c7906-ce72-4433-94df-441beb14d96a",
  "contents": "Buy groceries",
  "isCompleted": false
}
```

**Error Response:** `400 Bad Request`
```json
{
  "error": "Bad Request",
  "status": 400,
  "details": "ToDo content cannot be empty"
}
```

#### Toggle ToDo Completion
```http
POST /api/todos/:id/toggle
```
Toggles the completion status of a ToDo item.

**Response:** `204 No Content` | `404 Not Found` | `400 Bad Request`

#### Delete ToDo Item
```http
DELETE /api/todos/:id
```
Soft deletes a ToDo item (marks as deleted but doesn't remove from storage).

**Response:** `204 No Content` | `404 Not Found` | `400 Bad Request`

---

### Criminal Case Management API Endpoints

#### Create Criminal Case
```http
POST /api/cases
Content-Type: application/json

{
  "title": "Cyber Attack on Banking System",
  "description": "Unauthorized access to customer database detected",
  "crimeType": "cybercrime",
  "assignedJudge": "Judge Smith",
  "location": "New York, NY"
}
```

**Response:** `201 Created`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "caseNumber": "2024-123456",
  "title": "Cyber Attack on Banking System",
  "status": "open",
  "priority": "medium",
  "openedAt": "2024-01-15T10:30:00Z"
}
```

#### Get Case by ID
```http
GET /api/cases/:id
```

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "caseNumber": "2024-123456",
  "title": "Cyber Attack on Banking System",
  "description": "Unauthorized access to customer database",
  "crimeType": "cybercrime",
  "status": "under_investigation",
  "priority": "high",
  "assignedJudge": "Judge Smith",
  "location": "New York, NY",
  "defendants": ["John Doe"],
  "evidence": ["Server logs", "IP addresses"],
  "notesCount": 3
}
```

#### Search Cases
```http
GET /api/cases?status=open&priority=high&page=1&limit=10
```

**Query Parameters:**
- `status`: Filter by case status (open, under_investigation, closed, cold_case)
- `priority`: Filter by priority (critical, high, medium, low)
- `judge`: Filter by assigned judge name
- `active`: Filter active cases only (true/false)
- `page`: Page number for pagination
- `limit`: Items per page

**Response:** `200 OK`
```json
{
  "cases": [...],
  "total": 42
}
```

#### Get Case Statistics
```http
GET /api/cases/statistics
```

**Response:** `200 OK`
```json
{
  "totalCases": 150,
  "openCases": 45,
  "underInvestigation": 30,
  "closedCases": 60,
  "coldCases": 15,
  "criticalPriority": 5,
  "highPriority": 20
}
```

#### Update Case Status
```http
PATCH /api/cases/:id/status
Content-Type: application/json

{
  "status": "under_investigation"
}
```

**Valid Status Values:** `open`, `under_investigation`, `on_hold`, `closed`, `cold_case`

**Response:** `200 OK` - Returns updated case

#### Update Case Priority
```http
PATCH /api/cases/:id/priority
Content-Type: application/json

{
  "priority": "high"
}
```

**Valid Priority Values:** `low`, `medium`, `high`, `critical`

**Response:** `200 OK` - Returns updated case

#### Add Defendant to Case
```http
POST /api/cases/:id/defendants
Content-Type: application/json

{
  "name": "John Doe"
}
```

**Response:** `200 OK` - Returns updated case with new defendant

#### Add Evidence to Case
```http
POST /api/cases/:id/evidence
Content-Type: application/json

{
  "description": "Security camera footage from 2024-01-15"
}
```

**Response:** `200 OK` - Returns updated case with new evidence

#### Add Note to Case
```http
POST /api/cases/:id/notes
Content-Type: application/json

{
  "content": "Witness interview completed",
  "author": "Court Clerk"
}
```

**Response:** `200 OK` - Returns updated case with new note

#### Get Case by Case Number
```http
GET /api/cases/by-number/:case_number
```

**Response:** `200 OK` - Returns the case with the specified case number

#### Get Cases by Judge
```http
GET /api/cases/by-judge/:judge
```

**Response:** `200 OK` - Returns all cases assigned to the specified judge

#### Count Cases by Status
```http
GET /api/cases/count-by-status/:status
```

**Response:** `200 OK`
```json
{
  "count": 15,
  "status": "open"
}
```

#### Delete Case
```http
DELETE /api/cases/:id
```

**Response:** `204 No Content` - Case successfully deleted

## Prerequisites

To build and run the Spin application on your local machine, you must have:

- **Spin CLI** version `3.3.1` or newer
  - Install: `curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash`
- **Rust** version `1.86.0` or newer with the `wasm32-wasip1` target
  - Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Add WASM target: `rustup target add wasm32-wasip1`

## Building & Running

### Build the Application
```bash
spin build
```

### Development Mode (With Hot Reload)
```bash
# Development with hot reload and sample districts
./dev.sh
```
- Uses `spin watch` with runtime config for hot reload
- Includes 3 sample districts for testing (SDNY, EDNY, NDCA)
- Use header: `X-Court-District: SDNY` (or EDNY, NDCA)
- Default store reserved for global configuration

### Production Mode (Multi-Tenant)
```bash
# Production with all 94 federal district stores
./prod.sh
```
- Full multi-tenant configuration with physical isolation
- 94 separate database files (one per federal district)
- Use header: `X-Court-District: SDNY` (or any district code)

### Testing Multi-Tenant Isolation
```bash
# Create attorney in SDNY district
curl -X POST http://localhost:3000/api/attorneys \
  -H "X-Court-District: sdny" \
  -H "Content-Type: application/json" \
  -d '{"first_name":"John","last_name":"Smith",...}'

# Create attorney in EDNY district (completely isolated)
curl -X POST http://localhost:3000/api/attorneys \
  -H "X-Court-District: edny" \
  -H "Content-Type: application/json" \
  -d '{"first_name":"Jane","last_name":"Doe",...}'
```

### Deploy to Fermyon Cloud
```bash
spin deploy
```

## Project Structure

```
federal-judicial-cms/
├── src/
│   ├── lib.rs                          # Main application entry point and comprehensive router
│   ├── error.rs                        # Custom error types for judicial operations
│   ├── utils/                          # Utility functions and helpers
│   │   └── mod.rs                     # Utility module exports
│   ├── domain/                         # Business domain models (hexagonal core)
│   │   ├── mod.rs                     # Domain module exports
│   │   ├── todo.rs                    # Legacy ToDo entity (demo/testing)
│   │   ├── criminal_case.rs           # Federal criminal case domain model
│   │   ├── judge.rs                   # Federal judge domain model
│   │   ├── docket.rs                  # Court docket and calendar domain
│   │   ├── deadline.rs                # Federal deadline tracking domain
│   │   └── features.rs                # Feature flag domain model
│   ├── ports/                          # Hexagonal architecture interfaces
│   │   ├── mod.rs                     # Ports module exports
│   │   ├── case_repository.rs         # Criminal case repository trait
│   │   ├── judge_repository.rs        # Judge management repository trait
│   │   ├── docket_repository.rs       # Docket & calendar repository trait
│   │   ├── deadline_repository.rs     # Deadline tracking repository trait
│   │   └── feature_repository.rs      # Feature flag repository trait
│   ├── adapters/                       # Infrastructure implementations
│   │   ├── mod.rs                     # Adapters module exports
│   │   ├── spin_kv_case_repository.rs # Criminal case Spin KV implementation
│   │   ├── spin_kv_judge_repository.rs # Judge management Spin KV implementation
│   │   ├── spin_kv_docket_repository.rs # Docket & calendar Spin KV implementation
│   │   ├── spin_kv_deadline_repository.rs # Deadline tracking Spin KV implementation
│   │   └── spin_kv_feature_repository.rs # Feature flag Spin KV implementation
│   └── handlers/                       # HTTP request handlers (200+ endpoints)
│       ├── mod.rs                     # Handler module exports
│       ├── todo.rs                    # Legacy ToDo handlers (demo/testing)
│       ├── criminal_case.rs           # Federal case management (20+ endpoints)
│       ├── judge.rs                   # Judge management system (26 endpoints)
│       ├── docket.rs                  # Docket & calendar management (28 endpoints)
│       ├── deadline.rs                # Deadline tracking & compliance (26 endpoints)
│       ├── features.rs                # Feature flag management (10 endpoints)
│       ├── admin.rs                   # Multi-tenant administration (2 endpoints)
│       ├── health.rs                  # System health monitoring
│       └── docs.rs                    # Comprehensive OpenAPI documentation
├── spin.toml                           # Spin application configuration
├── Cargo.toml                          # Rust dependencies and metadata
└── README.md                           # Comprehensive system documentation
```

### Module Responsibilities

#### **Domain Layer (Pure Business Logic)**
- **criminal_case.rs**: Federal criminal case lifecycle, plea management, motion tracking
- **judge.rs**: Judge profiles, workload management, conflict of interest, recusal system
- **docket.rs**: Electronic filing, court scheduling, calendar management, Speedy Trial Act
- **deadline.rs**: FRCP compliance, deadline calculation, extension management, reminders
- **features.rs**: Feature flag management, implementation tracking, A/B testing

#### **Ports Layer (Interfaces)**
- Repository traits defining contracts for each domain
- Service interfaces for external integrations
- Event publishing interfaces for audit logging

#### **Adapters Layer (Infrastructure)**
- Spin KV store implementations for all repositories
- External service adapters (future: PostgreSQL, MongoDB, etc.)
- Event publishers for audit trails and notifications

#### **Handlers Layer (HTTP API)**
- 128+ REST endpoints across 8 major modules
- Comprehensive input validation and error handling
- OpenAPI documentation with examples and schemas
- Multi-tenant request processing and authorization

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## Architecture

This federal judicial case management system follows **Hexagonal Architecture** (Ports and Adapters) with enterprise-grade design patterns:

### Core Architecture Layers

- **Domain Layer** (`src/domain/`): Pure business logic for all judicial processes (cases, judges, deadlines, dockets)
- **Ports Layer** (`src/ports/`): Interfaces defining contracts between domain and infrastructure
- **Adapters Layer** (`src/adapters/`): Infrastructure implementations (Spin KV store, external services)
- **Handlers Layer** (`src/handlers/`): HTTP request handlers with comprehensive input validation
- **Error Layer** (`src/error.rs`): Centralized error handling with typed judicial-specific errors

### Enterprise Design Patterns

#### 1. **Hexagonal Architecture Implementation**
```
┌─────────────────────────────────────────────┐
│              HTTP Handlers                  │  ← Inbound Adapters
│    (case, judge, docket, deadline mgmt)     │
├─────────────────────────────────────────────┤
│              Repository Ports               │  ← Ports (Interfaces)
│         (case, judge, docket repos)         │
├─────────────────────────────────────────────┤
│             Domain Models                   │  ← Core Business Logic
│  (CriminalCase, Judge, Docket, Deadline)    │
├─────────────────────────────────────────────┤
│          Spin KV Repositories               │  ← Outbound Adapters
│      (spin_kv_*_repository.rs files)        │
└─────────────────────────────────────────────┘
```

#### 2. **Multi-Tenant Data Architecture**
- **Tenant Isolation**: Complete data separation by tenant ID
- **Namespace Segregation**: All storage keys prefixed with tenant identifiers
- **Resource Quotas**: Per-tenant resource limits and monitoring
- **Feature Gating**: Tenant-specific feature enablement

#### 3. **Federal Judicial Domain Modeling**
- **Case Lifecycle Management**: Complete criminal case workflow from filing to disposition
- **Judge Assignment Algorithm**: Conflict-aware automatic case assignment
- **Deadline Compliance**: FRCP and federal statute deadline tracking
- **Court Calendar**: Resource-optimized scheduling with conflict detection

### Key Design Decisions

#### **Enterprise Reliability**
- **Atomic Operations**: All critical updates use atomic transactions
- **Idempotent Operations**: Safe retry mechanisms for all endpoints
- **Soft Deletes**: Audit trail preservation for judicial records
- **UUID Identifiers**: Globally unique identifiers across all entities
- **Comprehensive Validation**: Multi-layer input validation with judicial rule enforcement

#### **Performance & Scalability**
- **Pagination**: Efficient handling of large judicial datasets
- **Caching Strategy**: Intelligent caching for frequently accessed judicial data
- **Query Optimization**: Optimized searches across cases, judges, and dockets
- **Resource Pooling**: Efficient courtroom and judge resource allocation

#### **Compliance & Security**
- **Audit Logging**: Complete audit trail for all judicial actions
- **Access Control**: Role-based access control for court personnel
- **Data Retention**: Federal record retention policy compliance
- **Privacy Protection**: Sealed document and sensitive information handling

### Storage Architecture

The system uses a sophisticated storage strategy with Spin's key-value store:

#### **Namespace Organization**
```
tenant:{tenant_id}:case:{case_id}          # Criminal cases
tenant:{tenant_id}:judge:{judge_id}        # Judge profiles
tenant:{tenant_id}:docket:{case_id}        # Docket entries
tenant:{tenant_id}:deadline:{deadline_id}  # Deadline tracking
tenant:{tenant_id}:calendar:{event_id}     # Court events
tenant:{tenant_id}:features:{feature_path} # Feature flags
```

#### **Data Consistency Features**
- **ACID Transactions**: Atomic updates for critical judicial operations
- **Conflict Resolution**: Optimistic locking for concurrent updates
- **Backup & Recovery**: Automated backup strategies for judicial data
- **Cross-Reference Integrity**: Maintained relationships between cases, judges, and events

#### **Performance Optimizations**
- **Index Structures**: Efficient lookups by case number, judge name, date ranges
- **Batch Operations**: Bulk updates for large-scale judicial data processing
- **Compression**: Efficient storage of large judicial documents and evidence
- **Lazy Loading**: On-demand loading of detailed judicial information

## API Error Handling

All error responses follow a consistent JSON structure:
```json
{
  "error": "Error Type",
  "status": 400,
  "details": "Detailed error message"
}
```

**Common Error Codes:**
- `400 Bad Request`: Invalid input data or parameters
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server-side errors
- `503 Service Unavailable`: Storage connectivity issues

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).

## Author

Tyler Harpool - [GitHub](https://github.com/tyler-harpool)

## Testing the Federal Judicial Lexodus System

### Using cURL Examples

The system provides 200+ REST endpoints. Here are key examples for testing major functionality:

#### **System Health & Status**
```bash
# Check system health
curl "http://localhost:3000/api/health"

# Get tenant statistics (multi-tenancy)
curl "http://localhost:3000/api/admin/tenant-stats"
```

#### **Federal Case Management**
```bash
# Create a new criminal case
curl -X POST "http://localhost:3000/api/cases" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "United States v. Doe",
    "description": "Federal fraud charges",
    "crimeType": "fraud",
    "assignedJudge": "Judge Smith",
    "location": "Northern District of California"
  }'

# Enter a defendant plea
curl -X POST "http://localhost:3000/api/cases/{case_id}/plea" \
  -H "Content-Type: application/json" \
  -d '{
    "defendant": "John Doe",
    "pleaType": "not_guilty",
    "pleaDate": "2024-01-20T14:30:00Z"
  }'

# Get case statistics
curl "http://localhost:3000/api/cases/statistics"
```

#### **Judge Management**
```bash
# Create judge profile
curl -X POST "http://localhost:3000/api/judges" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Hon. Patricia Johnson",
    "title": "District Judge",
    "district": "Northern District of California",
    "status": "active"
  }'

# Check for conflicts of interest
curl "http://localhost:3000/api/judges/conflicts/check/Acme%20Corp"

# Get judge workload statistics
curl "http://localhost:3000/api/judges/workload"
```

#### **Deadline Tracking & FRCP Compliance**
```bash
# Calculate FRCP deadlines
curl -X POST "http://localhost:3000/api/deadlines/calculate" \
  -H "Content-Type: application/json" \
  -d '{
    "triggerEvent": "service_of_process",
    "triggerDate": "2024-01-15",
    "ruleType": "FRCP",
    "specificRule": "12(b)"
  }'

# Get compliance statistics
curl "http://localhost:3000/api/compliance/stats"
```

#### **Speedy Trial Act Compliance**
```bash
# Get approaching Speedy Trial deadlines
curl "http://localhost:3000/api/speedy-trial/approaching"

# Check for violations
curl "http://localhost:3000/api/speedy-trial/violations"
```

#### **Feature Flag Management**
```bash
# Get all feature flags
curl "http://localhost:3000/api/features"

# Toggle a feature
curl -X PATCH "http://localhost:3000/api/features" \
  -H "Content-Type: application/json" \
  -d '{"path": "case_management.advanced_search", "enabled": true}'
```

#### **Legacy ToDo System (Demo/Testing)**
```bash
# Create a ToDo (legacy functionality)
curl -X POST "http://localhost:3000/api/todos" \
  -H "Content-Type: application/json" \
  -d '{"contents": "Test judicial workflow"}'

# Get paginated todos
curl "http://localhost:3000/api/todos?page=1&limit=5"
```

### Using Swagger UI

Navigate to `http://localhost:3000/docs` for an interactive API explorer where you can:
- View all endpoint documentation
- Test API calls directly from the browser
- See request/response schemas
- Download the OpenAPI specification

## Changelog

### Version 4.0.0 (Current) - Major Release: Complete Federal Judicial Lexodus System
**🏛️ Massive System Expansion: From Simple API to Enterprise Federal Court Management**

This release represents a complete transformation from a simple ToDo API to a comprehensive federal judicial case management system equivalent to Lexodus (Case Management/Electronic Case Files) used in federal courts.

#### **🏗️ Architecture Overhaul**
- **Complete Hexagonal Architecture**: Implemented across all 8 major system modules
- **Multi-Tenant Enterprise Architecture**: Full tenant isolation and resource management
- **Repository Pattern**: Clean separation between domain logic and infrastructure across all modules
- **Domain-Driven Design**: Federal judicial domain modeling with proper business logic encapsulation

#### **⚖️ Federal Case Management System (20+ Endpoints)**
- **Complete Criminal Case Lifecycle**: Filing through sentencing with full federal workflow
- **Advanced Plea Management**: Support for all federal plea types (Guilty, Not Guilty, Nolo Contendere, Alford)
- **Federal Motion System**: Complete pretrial and post-trial motion filing and ruling workflow
- **Court Event Scheduling**: Arraignments, hearings, trials, and sentencing with resource management
- **Defendant Management**: Multi-defendant case support with individual tracking
- **Evidence Tracking**: Comprehensive evidence management with chain of custody
- **Case Statistics**: Real-time analytics and federal case metrics
- **Advanced Search**: Multi-criteria search with complex filtering capabilities

#### **👨‍⚖️ Judge Management System (26 Endpoints)**
- **Comprehensive Judge Profiles**: Complete federal judge information and credentials management
- **Intelligent Case Assignment**: Automated assignment with conflict detection and workload balancing
- **Conflict of Interest System**: Advanced conflict tracking and resolution workflows
- **Recusal Management**: Complete recusal filing, review, and approval processes
- **Workload Analytics**: Judge productivity metrics and case completion statistics
- **Availability Management**: Judge scheduling, vacation, and district assignment tracking
- **Performance Metrics**: Comprehensive judge performance and efficiency analytics

#### **📋 Docket & Calendar Management (28 Endpoints)**
- **Electronic Docket System**: Complete docket entry management with attachment support
- **Advanced Court Scheduling**: Resource-optimized scheduling with conflict detection
- **Courtroom Management**: Resource allocation and utilization optimization
- **Professional Docket Sheets**: Federal-compliant docket sheet generation and formatting
- **Sealed Document Handling**: Secure management of confidential and sensitive filings
- **Filing Statistics**: Comprehensive analytics for court filing trends and patterns
- **Search Capabilities**: Advanced search across all court filings and documents

#### **⏰ Deadline Tracking & Compliance (26 Endpoints)**
- **FRCP Deadline Calculator**: Automated Federal Rules of Civil Procedure deadline calculations
- **Comprehensive Deadline Management**: Multi-layered deadline tracking with escalation procedures
- **Federal Compliance Monitoring**: Real-time compliance tracking and violation detection
- **Automated Reminder System**: Multi-channel reminders with acknowledgment tracking
- **Extension Management**: Complete extension request and approval workflow
- **Performance Analytics**: Deadline compliance statistics and risk assessment
- **Jurisdictional Deadline Tracking**: Special handling for critical federal deadlines

#### **⏱️ Speedy Trial Act Compliance (8 Endpoints)**
- **70-Day Clock Management**: Automated Speedy Trial Act timeline tracking and compliance
- **Excludable Delay Tracking**: Comprehensive categorization and documentation of delays
- **Violation Detection**: Automatic detection and reporting of Speedy Trial violations
- **Federal Reporting Compliance**: Complete federal reporting requirements support
- **Critical Deadline Management**: Escalation and notification for approaching deadlines

#### **🎛️ Feature Flag Management (10 Endpoints)**
- **Enterprise Feature Control**: Dynamic feature enabling/disabling with rollout management
- **Implementation Lifecycle**: Complete feature development and deployment tracking
- **A/B Testing Framework**: Experimental feature testing with statistical analysis
- **Access Control**: Role-based feature access and permission management
- **Usage Analytics**: Feature adoption metrics and usage pattern analysis

#### **🏢 Multi-Tenant Administration (2 Endpoints)**
- **Enterprise Multi-Tenancy**: Complete tenant lifecycle management with data isolation
- **Resource Management**: Per-tenant resource usage tracking and billing support
- **Scalability**: Enterprise-grade multi-tenant architecture with performance monitoring
- **Data Security**: Secure tenant data separation with audit capabilities

#### **📊 System Enhancements**
- **200+ REST API Endpoints**: Comprehensive API coverage across all judicial functions
- **Enterprise-Grade Error Handling**: Sophisticated error management with judicial-specific error types
- **Advanced OpenAPI Documentation**: Complete documentation with examples and testing capabilities
- **Performance Optimization**: Efficient handling of large-scale judicial datasets
- **Audit Trail System**: Complete audit logging for all judicial operations
- **Security Enhancements**: Role-based access control and sensitive information protection

#### **🔧 Technical Improvements**
- **Atomic Operations**: ACID transaction support for critical judicial operations
- **Advanced Caching**: Intelligent caching strategies for frequently accessed data
- **Query Optimization**: Optimized searches across all judicial entities
- **Namespace Organization**: Sophisticated multi-tenant data organization
- **Backup & Recovery**: Automated backup strategies for judicial data preservation

### Version 3.0.0
- Added Criminal Case Management API demonstrating Hexagonal Architecture
- Implemented repository pattern with ports and adapters
- Added case statistics endpoint
- Support for case status and priority management
- Comprehensive search and filtering for cases
- Clean separation between domain logic and infrastructure

### Version 2.0.0
- Added pagination support for listing ToDos
- Added filtering by completion status
- Implemented comprehensive error handling with typed errors
- Added health check endpoint for monitoring
- Enhanced input validation with meaningful error messages
- Improved OpenAPI documentation with complete schemas
- Added request/response examples in documentation

### Version 1.0.0
- Initial release with basic CRUD operations
- Soft delete functionality
- UUID-based identification
- OpenAPI documentation

## PDF Document Generation API

### Overview
The PDF generation system demonstrates hexagonal architecture with clean separation between business logic and infrastructure. It supports Supreme Court Rule 33 formatting standards for professional legal documents.

### Architecture
```
┌─────────────────────────────────────────────┐
│          HTTP Handlers                      │  ← Inbound Adapters
│      (handlers/pdf_hexagonal.rs)            │
├─────────────────────────────────────────────┤
│          PDF Service                        │  ← Application Layer
│      (services/pdf_service.rs)              │
├─────────────────────────────────────────────┤
│       Document Generator Port               │  ← Port (Interface)
│    (ports/document_generator.rs)            │
├─────────────────────────────────────────────┤
│          Domain Models                      │  ← Core Business Logic
│        (domain/document.rs)                 │
├─────────────────────────────────────────────┤
│       PDF Writer Adapter                    │  ← Outbound Adapter
│   (adapters/pdf_writer_adapter.rs)          │
└─────────────────────────────────────────────┘
```

### Response Format Selection

**NEW in v5.1.0:** All PDF endpoints now use explicit URL-based format selection for clarity:
- **`/api/pdf/{endpoint}/json`** → Returns JSON with base64-encoded PDF
- **`/api/pdf/{endpoint}/pdf`** → Returns raw PDF binary for direct download
- **`/api/pdf/{endpoint}`** → Defaults to JSON format for backward compatibility

This approach is more explicit and works better with API documentation tools like Swagger UI.

### Key Endpoints

#### Generate Rule 16(b) Order
```bash
# Generate with JSON response (base64 PDF)
curl -X POST "http://localhost:3000/api/pdf/rule16b/json" \
  -H "Content-Type: application/json" \
  -H "X-Court-District: SDNY" \
  -d '{
    "case_number": "2024-CR-00123",
    "defendant_names": "John Doe",
    "judge_name": "Hon. Patricia Johnson",
    "trial_date": "2024-06-15",
    "discovery_deadline": "2024-04-01",
    "motion_deadline": "2024-05-01",
    "pretrial_conference_date": "2024-05-15"
  }'

# Generate with PDF response (direct download)
curl -X POST "http://localhost:3000/api/pdf/rule16b/pdf" \
  -H "Content-Type: application/json" \
  -H "X-Court-District: SDNY" \
  -d '{
    "case_number": "2024-CR-00123",
    "defendant_names": "John Doe",
    "judge_name": "Hon. Patricia Johnson"
  }' --output rule16b-order.pdf
```

#### Generate Signed Documents
```bash
# Generate signed Rule 16(b) with stored signature - JSON format
curl -X POST "http://localhost:3000/api/pdf/signed/rule16b/json" \
  -H "Content-Type: application/json" \
  -H "X-Court-District: SDNY" \
  -d '{
    "case_number": "2024-CR-00123",
    "defendant_names": "John Doe",
    "judge_name": "Hon. Patricia Johnson",
    "judge_id": "550e8400-e29b-41d4-a716-446655440000"
  }'

# Generate signed Rule 16(b) - PDF format
curl -X POST "http://localhost:3000/api/pdf/signed/rule16b/pdf" \
  -H "Content-Type: application/json" \
  -H "X-Court-District: SDNY" \
  -d '{
    "case_number": "2024-CR-00123",
    "defendant_names": "John Doe",
    "judge_name": "Hon. Patricia Johnson",
    "judge_id": "550e8400-e29b-41d4-a716-446655440000"
  }' --output signed-order.pdf
```

#### Batch Document Generation
```bash
# Generate multiple documents at once
curl -X POST "http://localhost:3000/api/pdf/batch" \
  -H "Content-Type: application/json" \
  -d '{
    "documents": [
      {
        "type": "rule16b_order",
        "case_number": "2024-CR-00123",
        "defendant_name": "John Doe",
        "judge_name": "Hon. Patricia Johnson",
        "trial_date": "2024-06-15"
      },
      {
        "type": "minute_entry",
        "case_number": "2024-CR-00456",
        "hearing_type": "Arraignment",
        "date": "2024-01-20"
      }
    ]
  }'
```

#### Signature Management
```bash
# Store a judge's signature
curl -X POST "http://localhost:3000/api/signatures" \
  -H "Content-Type: application/json" \
  -d '{
    "judge_id": "550e8400-e29b-41d4-a716-446655440000",
    "signature_base64": "iVBORw0KGgoAAAANS..."
  }'

# Retrieve a judge's signature
curl "http://localhost:3000/api/signatures/550e8400-e29b-41d4-a716-446655440000"
```

### Document Formatting Features
- **Supreme Court Rule 33 Compliance**: Professional legal document formatting
- **8.5x11" Letter Size**: Standard U.S. court document size
- **Times-Roman Font**: Traditional legal document typography
- **1-inch Margins**: Standard court filing margins
- **Double-Spacing**: Enhanced readability for legal documents
- **Text Wrapping**: Automatic text wrapping to prevent margin overflow
- **Centered Headers**: Professional document layout
- **Signature Boxes**: Bordered electronic signature areas

### Response Format Selection
The API automatically selects the response format based on the `Accept` header:
- `Accept: application/pdf` → Returns raw PDF binary
- `Accept: application/json` → Returns JSON with base64-encoded PDF
- Default (no Accept header) → Returns JSON with base64-encoded PDF

## Acknowledgments

- Built with [Fermyon Spin](https://www.fermyon.com/spin)
- Documentation powered by [utoipa](https://github.com/juhaku/utoipa)
- Based on the guide: [OpenAPI Docs for Spin with Rust](https://www.fermyon.com/blog/openapi-docs-for-spin-with-rust)
