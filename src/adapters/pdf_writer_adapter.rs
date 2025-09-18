use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
use async_trait::async_trait;
use crate::domain::document::{
    CourtDocument, GeneratedDocument, DocumentError, DocumentMetadata,
    CaseNumber, JudgeName, District, ElectronicSignature
};
use crate::ports::document_generator::{DocumentGenerator, PdfRenderer};

pub struct PdfWriterAdapter;

impl PdfWriterAdapter {
    pub fn new() -> Self {
        Self
    }

    fn create_pdf_structure() -> (Pdf, Ref, Ref) {
        let mut pdf = Pdf::new();
        let page_tree_id = Ref::new(2);
        let page_id = Ref::new(3);
        let font_id = Ref::new(4);
        let content_id = Ref::new(5);
        let font_name = Name(b"F1");

        pdf.catalog(Ref::new(1)).pages(page_tree_id);
        pdf.pages(page_tree_id).kids([page_id]).count(1);

        let mut page = pdf.page(page_id);
        page.parent(page_tree_id);
        page.contents(content_id);
        // 8.5 x 11 inches in points (72 points per inch)
        let width = 612.0;  // 8.5 * 72
        let height = 792.0; // 11 * 72
        page.media_box(Rect::new(0.0, 0.0, width, height));
        page.resources().fonts().pair(font_name, font_id);
        page.finish();

        // Use Times-Roman for more formal legal documents
        pdf.type1_font(font_id).base_font(Name(b"Times-Roman"));

        (pdf, page_id, content_id)
    }

    fn add_header(content: &mut Content, district: &str) -> f32 {
        // Start with 1 inch margin from top (72 points)
        let mut y_position = 720.0; // 792 - 72 = 720

        // Calculate center of page (612 / 2 = 306)
        let page_center = 306.0;

        // Approximate text widths for centering
        let court_text_width = 180.0; // "UNITED STATES DISTRICT COURT" width
        let district_width = 30.0; // "sdny" width estimate

        // Center the header text
        content.begin_text();
        content.set_font(Name(b"F1"), 12.0); // Standard 12-point font
        content.next_line(page_center - court_text_width / 2.0, y_position);
        content.show(Str(b"UNITED STATES DISTRICT COURT"));
        content.end_text();

        y_position -= 24.0; // Double-spacing (12pt * 2)
        content.begin_text();
        content.set_font(Name(b"F1"), 12.0);
        content.next_line(page_center - district_width / 2.0, y_position);
        content.show(Str(district.as_bytes()));
        content.end_text();

        y_position - 40.0
    }

    fn add_case_caption(content: &mut Content, case_number: &str, defendant_names: &str, mut y_position: f32) -> f32 {
        y_position -= 48.0; // Double-spacing before caption

        // Left margin at 1 inch (72 points)
        let left_margin = 72.0;
        let case_num_x = 400.0; // Right side for case number

        content.begin_text();
        content.set_font(Name(b"F1"), 12.0);
        content.next_line(left_margin, y_position);
        content.show(Str(b"UNITED STATES OF AMERICA,"));
        content.end_text();

        y_position -= 24.0; // Double-spacing
        content.begin_text();
        content.next_line(left_margin + 20.0, y_position);
        content.show(Str(b"Plaintiff,"));
        content.end_text();

        y_position -= 24.0;
        content.begin_text();
        content.next_line(left_margin, y_position);
        content.show(Str(b"v."));
        content.end_text();

        content.begin_text();
        content.next_line(case_num_x, y_position);
        content.show(Str(case_number.as_bytes()));
        content.end_text();

        y_position -= 24.0;
        content.begin_text();
        content.next_line(left_margin, y_position);
        content.show(Str(defendant_names.as_bytes()));
        content.end_text();

        y_position -= 24.0;
        content.begin_text();
        content.next_line(left_margin + 20.0, y_position);
        content.show(Str(b"Defendant(s)."));
        content.end_text();

        y_position - 48.0
    }

    fn add_electronic_signature(content: &mut Content, signature: &ElectronicSignature, judge_name: &str, mut y_position: f32) -> f32 {
        y_position -= 40.0;

        // Draw a box around the signature area (aligned with right margin)
        // Page width is 612, right margin at 540 (612 - 72)
        let box_x = 320.0;  // Right-aligned signature box
        let box_y = y_position - 120.0;
        let box_width = 220.0;
        let box_height = 135.0;

        // Set line width and draw rectangle
        content.set_line_width(1.0);
        content.rect(box_x, box_y, box_width, box_height);
        content.stroke();

        // Note: pdf-writer doesn't support fill operations directly
        // We'll use just the border for now

        content.begin_text();
        content.set_font(Name(b"F1"), 10.0);
        content.next_line(350.0, y_position);
        content.show(Str(b"ELECTRONICALLY SIGNED"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(350.0, y_position);
        let timestamp = signature.signed_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        content.show(Str(timestamp.as_bytes()));
        content.end_text();

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 14.0); // Make signature slightly larger
        content.next_line(350.0, y_position);
        let initials = format!("/s/ {}", extract_initials(&signature.signer_name));
        content.show(Str(initials.as_bytes()));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 12.0);
        content.next_line(350.0, y_position);
        content.show(Str(b"_______________________________"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(350.0, y_position);
        content.show(Str(judge_name.as_bytes()));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(350.0, y_position);
        content.show(Str(b"United States District Judge"));
        content.end_text();

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 8.0);
        content.next_line(350.0, y_position);
        let doc_id = format!("Doc ID: {}", signature.verification_code);
        content.show(Str(doc_id.as_bytes()));
        content.end_text();

        y_position - 10.0 // Add some extra spacing after the box
    }

    fn add_standard_signature(content: &mut Content, judge_name: &str, mut y_position: f32) -> f32 {
        y_position -= 60.0;

        content.begin_text();
        content.set_font(Name(b"F1"), 12.0);
        content.next_line(350.0, y_position);
        content.show(Str(b"_______________________________"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(350.0, y_position);
        content.show(Str(judge_name.as_bytes()));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(350.0, y_position);
        content.show(Str(b"United States District Judge"));
        content.end_text();

        y_position
    }
}

fn extract_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_uppercase()
}

/// Wrap text to fit within page margins (approximately 65 characters per line)
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        if paragraph.len() <= max_chars {
            lines.push(paragraph.to_string());
        } else {
            let words: Vec<&str> = paragraph.split_whitespace().collect();
            let mut current_line = String::new();

            for word in words {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + word.len() + 1 <= max_chars {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    lines.push(current_line);
                    current_line = word.to_string();
                }
            }

            if !current_line.is_empty() {
                lines.push(current_line);
            }
        }
    }

    lines
}

impl PdfRenderer for PdfWriterAdapter {
    fn render_rule16b(
        &self,
        case_number: &CaseNumber,
        defendant_names: &str,
        judge_name: &JudgeName,
        district: &District,
        signature: Option<&ElectronicSignature>
    ) -> Result<Vec<u8>, DocumentError> {
        let (mut pdf, _page_id, content_id) = Self::create_pdf_structure();
        let mut content = Content::new();

        let mut y_position = Self::add_header(&mut content, district.as_str());
        y_position = Self::add_case_caption(&mut content, case_number.as_str(), defendant_names, y_position);

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 14.0);
        content.next_line(200.0, y_position);
        content.show(Str(b"SCHEDULING ORDER"));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        content.show(Str(b"Pursuant to Rule 16(b) of the Federal Rules of Criminal Procedure,"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"the following schedule is hereby established:"));
        content.end_text();

        y_position -= 30.0;
        let deadlines = [
            ("1. Discovery Completion:", "60 days from date of this order"),
            ("2. Motion Filing Deadline:", "90 days from date of this order"),
            ("3. Pretrial Conference:", "120 days from date of this order"),
            ("4. Trial Date:", "To be scheduled at pretrial conference"),
        ];

        for (label, date) in deadlines.iter() {
            content.begin_text();
            content.set_font(Name(b"F1"), 11.0);
            content.next_line(70.0, y_position);
            content.show(Str(label.as_bytes()));
            content.end_text();

            content.begin_text();
            content.next_line(250.0, y_position);
            content.show(Str(date.as_bytes()));
            content.end_text();

            y_position -= 20.0;
        }

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        content.show(Str(b"IT IS SO ORDERED."));
        content.end_text();

        if let Some(sig) = signature {
            Self::add_electronic_signature(&mut content, sig, judge_name.as_str(), y_position);
        } else {
            Self::add_standard_signature(&mut content, judge_name.as_str(), y_position);
        }

        pdf.stream(content_id, &content.finish());
        Ok(pdf.finish())
    }

    fn render_court_order(
        &self,
        case_number: &CaseNumber,
        defendant_names: &str,
        judge_name: &JudgeName,
        district: &District,
        order_title: &str,
        order_content: &str,
        signature: Option<&ElectronicSignature>
    ) -> Result<Vec<u8>, DocumentError> {
        let (mut pdf, _page_id, content_id) = Self::create_pdf_structure();
        let mut content = Content::new();

        let mut y_position = Self::add_header(&mut content, district.as_str());
        y_position = Self::add_case_caption(&mut content, case_number.as_str(), defendant_names, y_position);

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 14.0);
        content.next_line(200.0, y_position);
        content.show(Str(order_title.as_bytes()));
        content.end_text();

        y_position -= 30.0;
        for line in order_content.lines() {
            content.begin_text();
            content.set_font(Name(b"F1"), 11.0);
            content.next_line(50.0, y_position);
            content.show(Str(line.as_bytes()));
            content.end_text();
            y_position -= 15.0;
        }

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        content.show(Str(b"IT IS SO ORDERED."));
        content.end_text();

        if let Some(sig) = signature {
            Self::add_electronic_signature(&mut content, sig, judge_name.as_str(), y_position);
        } else {
            Self::add_standard_signature(&mut content, judge_name.as_str(), y_position);
        }

        pdf.stream(content_id, &content.finish());
        Ok(pdf.finish())
    }

    fn render_minute_entry(
        &self,
        case_number: &CaseNumber,
        defendant_names: &str,
        judge_name: &JudgeName,
        district: &District,
        minute_text: &str
    ) -> Result<Vec<u8>, DocumentError> {
        let (mut pdf, _page_id, content_id) = Self::create_pdf_structure();
        let mut content = Content::new();

        let mut y_position = Self::add_header(&mut content, district.as_str());
        y_position = Self::add_case_caption(&mut content, case_number.as_str(), defendant_names, y_position);

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 14.0);
        content.next_line(260.0, y_position); // Center "MINUTE ENTRY"
        content.show(Str(b"MINUTE ENTRY"));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        let date = chrono::Utc::now().format("%B %d, %Y").to_string();
        content.show(Str(date.as_bytes()));
        content.end_text();

        y_position -= 20.0;
        // Wrap text to fit within margins (approximately 80-85 characters per line)
        // With 1-inch margins on 8.5" paper, we have 6.5" = 468 points of usable width
        // At 11pt font, we can fit about 85 characters comfortably
        let wrapped_lines = wrap_text(minute_text, 85);
        let left_margin = 72.0; // 1 inch margin

        for line in wrapped_lines {
            content.begin_text();
            content.set_font(Name(b"F1"), 11.0);
            content.next_line(left_margin, y_position);
            content.show(Str(line.as_bytes()));
            content.end_text();
            y_position -= 18.0; // 1.5x line spacing
        }

        y_position -= 30.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        let clerk_text = format!("Entered by: Deputy Clerk, {}", judge_name.as_str());
        content.show(Str(clerk_text.as_bytes()));
        content.end_text();

        pdf.stream(content_id, &content.finish());
        Ok(pdf.finish())
    }

    fn render_waiver_indictment(
        &self,
        case_number: &CaseNumber,
        defendant_name: &str,
        district: &District,
        charges: &str
    ) -> Result<Vec<u8>, DocumentError> {
        let (mut pdf, _page_id, content_id) = Self::create_pdf_structure();
        let mut content = Content::new();

        let mut y_position = Self::add_header(&mut content, district.as_str());

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 14.0);
        content.next_line(150.0, y_position);
        content.show(Str(b"WAIVER OF INDICTMENT"));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        content.show(Str(b"Case Number:"));
        content.end_text();

        content.begin_text();
        content.next_line(150.0, y_position);
        content.show(Str(case_number.as_str().as_bytes()));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        let text = format!("I, {}, having been advised of the nature of the charge(s):", defendant_name);
        content.show(Str(text.as_bytes()));
        content.end_text();

        y_position -= 20.0;
        content.begin_text();
        content.next_line(70.0, y_position);
        content.show(Str(charges.as_bytes()));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"and of my rights to:"));
        content.end_text();

        let rights = [
            "1. Have the charge(s) presented to a grand jury",
            "2. Assistance of counsel at all stages",
            "3. Remain silent",
            "4. A speedy and public trial",
        ];

        y_position -= 20.0;
        for right in rights.iter() {
            content.begin_text();
            content.next_line(70.0, y_position);
            content.show(Str(right.as_bytes()));
            content.end_text();
            y_position -= 15.0;
        }

        y_position -= 20.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"hereby waive prosecution by indictment and consent to prosecution"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"by information."));
        content.end_text();

        y_position -= 40.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"_______________________________"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"Defendant's Signature"));
        content.end_text();

        content.begin_text();
        content.next_line(300.0, y_position + 15.0);
        content.show(Str(b"Date: ________________"));
        content.end_text();

        y_position -= 40.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"_______________________________"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"Attorney for Defendant"));
        content.end_text();

        pdf.stream(content_id, &content.finish());
        Ok(pdf.finish())
    }

    fn render_conditions_release(
        &self,
        case_number: &CaseNumber,
        defendant_name: &str,
        district: &District,
        judge_name: &JudgeName,
        conditions: &[String]
    ) -> Result<Vec<u8>, DocumentError> {
        let (mut pdf, _page_id, content_id) = Self::create_pdf_structure();
        let mut content = Content::new();

        let mut y_position = Self::add_header(&mut content, district.as_str());
        y_position = Self::add_case_caption(&mut content, case_number.as_str(), defendant_name, y_position);

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 14.0);
        content.next_line(150.0, y_position);
        content.show(Str(b"ORDER SETTING CONDITIONS OF RELEASE"));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        content.show(Str(b"Upon consideration of the factors set forth in 18 U.S.C. 3142,"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"the Court orders the defendant released on the following conditions:"));
        content.end_text();

        y_position -= 25.0;
        for (i, condition) in conditions.iter().enumerate() {
            content.begin_text();
            content.next_line(70.0, y_position);
            let numbered = format!("{}. {}", i + 1, condition);
            content.show(Str(numbered.as_bytes()));
            content.end_text();
            y_position -= 20.0;
        }

        y_position -= 20.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"Violation of any condition may result in immediate arrest,"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"revocation of release, and prosecution under 18 U.S.C. 3148."));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"IT IS SO ORDERED."));
        content.end_text();

        Self::add_standard_signature(&mut content, judge_name.as_str(), y_position);

        pdf.stream(content_id, &content.finish());
        Ok(pdf.finish())
    }

    fn render_criminal_judgment(
        &self,
        case_number: &CaseNumber,
        defendant_name: &str,
        district: &District,
        judge_name: &JudgeName,
        plea: &str,
        counts: &str,
        sentence: &str
    ) -> Result<Vec<u8>, DocumentError> {
        let (mut pdf, _page_id, content_id) = Self::create_pdf_structure();
        let mut content = Content::new();

        let mut y_position = Self::add_header(&mut content, district.as_str());
        y_position = Self::add_case_caption(&mut content, case_number.as_str(), defendant_name, y_position);

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 14.0);
        content.next_line(180.0, y_position);
        content.show(Str(b"JUDGMENT IN A CRIMINAL CASE"));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        content.show(Str(b"The defendant:"));
        content.end_text();

        y_position -= 20.0;
        content.begin_text();
        content.next_line(70.0, y_position);
        let plea_text = format!("[ ] pleaded guilty to count(s) {}", if plea == "guilty" { counts } else { "" });
        content.show(Str(plea_text.as_bytes()));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(70.0, y_position);
        let nolo_text = format!("[ ] pleaded nolo contendere to count(s) {}", if plea == "nolo" { counts } else { "" });
        content.show(Str(nolo_text.as_bytes()));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(70.0, y_position);
        let verdict_text = format!("[ ] was found guilty on count(s) {}", if plea == "verdict" { counts } else { "" });
        content.show(Str(verdict_text.as_bytes()));
        content.end_text();

        y_position -= 30.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 12.0);
        content.next_line(50.0, y_position);
        content.show(Str(b"IMPRISONMENT"));
        content.end_text();

        y_position -= 20.0;
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(50.0, y_position);
        content.show(Str(b"The defendant is sentenced to:"));
        content.end_text();

        y_position -= 20.0;
        for line in sentence.lines() {
            content.begin_text();
            content.next_line(70.0, y_position);
            content.show(Str(line.as_bytes()));
            content.end_text();
            y_position -= 15.0;
        }

        y_position -= 30.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"IT IS ORDERED that the defendant shall notify the United States"));
        content.end_text();

        y_position -= 15.0;
        content.begin_text();
        content.next_line(50.0, y_position);
        content.show(Str(b"Attorney for this district of any change of address."));
        content.end_text();

        Self::add_standard_signature(&mut content, judge_name.as_str(), y_position - 20.0);

        pdf.stream(content_id, &content.finish());
        Ok(pdf.finish())
    }
}

impl PdfWriterAdapter {
    pub fn generate_document_sync(&self, document: CourtDocument) -> Result<GeneratedDocument, DocumentError> {
        let pdf_data = match &document.metadata {
            DocumentMetadata::Rule16b { defendant_names, judge_name, signature } => {
                self.render_rule16b(
                    &document.case_number,
                    defendant_names,
                    judge_name,
                    &document.district,
                    signature.as_ref()
                )?
            },
            DocumentMetadata::CourtOrder { defendant_names, judge_name, order_title, order_content, signature } => {
                self.render_court_order(
                    &document.case_number,
                    defendant_names,
                    judge_name,
                    &document.district,
                    order_title,
                    order_content,
                    signature.as_ref()
                )?
            },
            DocumentMetadata::MinuteEntry { defendant_names, judge_name, minute_text } => {
                self.render_minute_entry(
                    &document.case_number,
                    defendant_names,
                    judge_name,
                    &document.district,
                    minute_text
                )?
            },
            DocumentMetadata::WaiverIndictment { defendant_name, charges } => {
                self.render_waiver_indictment(
                    &document.case_number,
                    defendant_name,
                    &document.district,
                    charges
                )?
            },
            DocumentMetadata::ConditionsRelease { defendant_name, judge_name, conditions } => {
                self.render_conditions_release(
                    &document.case_number,
                    defendant_name,
                    &document.district,
                    judge_name,
                    conditions
                )?
            },
            DocumentMetadata::CriminalJudgment { defendant_name, judge_name, plea, counts, sentence } => {
                self.render_criminal_judgment(
                    &document.case_number,
                    defendant_name,
                    &document.district,
                    judge_name,
                    plea,
                    counts,
                    sentence
                )?
            },
        };

        let filename = format!("{}-{}.pdf",
            match document.document_type {
                crate::domain::document::DocumentType::Rule16b => "rule16b",
                crate::domain::document::DocumentType::CourtOrder => "court-order",
                crate::domain::document::DocumentType::MinuteEntry => "minute-entry",
                crate::domain::document::DocumentType::WaiverIndictment => "waiver-indictment",
                crate::domain::document::DocumentType::ConditionsRelease => "conditions-release",
                crate::domain::document::DocumentType::CriminalJudgment => "criminal-judgment",
            },
            document.case_number.as_str()
        );

        Ok(GeneratedDocument {
            document,
            pdf_data,
            filename,
        })
    }

    pub fn generate_batch_sync(&self, documents: Vec<CourtDocument>) -> Result<Vec<GeneratedDocument>, DocumentError> {
        let mut results = Vec::new();
        for document in documents {
            results.push(self.generate_document_sync(document)?);
        }
        Ok(results)
    }
}

#[async_trait]
impl DocumentGenerator for PdfWriterAdapter {
    async fn generate_document(&self, document: CourtDocument) -> Result<GeneratedDocument, DocumentError> {
        self.generate_document_sync(document)
    }

    async fn generate_batch(&self, documents: Vec<CourtDocument>) -> Result<Vec<GeneratedDocument>, DocumentError> {
        self.generate_batch_sync(documents)
    }
}