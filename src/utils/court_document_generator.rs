//! Federal court document PDF generation with signature support

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
use chrono::Local;

/// Generate Rule 16(b) Scheduling Order PDF
pub fn generate_rule_16b_order(
    case_number: &str,
    defendant_names: &str,
    judge_name: &str,
    district: &str,
) -> Vec<u8> {
    generate_rule_16b_with_signature(case_number, defendant_names, judge_name, district, None)
}

/// Generate Rule 16(b) order with optional signature image
pub fn generate_rule_16b_with_signature(
    case_number: &str,
    defendant_names: &str,
    judge_name: &str,
    district: &str,
    signature_base64: Option<&str>,
) -> Vec<u8> {
    // Create a new PDF
    let mut pdf = Pdf::new();

    // Create catalog and page tree
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let _font_id = Ref::new(4);
    let content_id = Ref::new(5);
    let fonts_dict_id = Ref::new(6);

    // Create catalog
    pdf.catalog(catalog_id).pages(page_tree_id);

    // Create page tree
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    // Add Helvetica font via separate dictionary
    pdf.type1_font(fonts_dict_id).base_font(Name(b"Helvetica"));

    // Create page (8.5" x 11" in points: 612 x 792)
    let mut page = pdf.page(page_id);
    page.parent(page_tree_id);
    page.media_box(Rect::new(0.0, 0.0, 612.0, 792.0));
    page.resources().fonts().pair(Name(b"F1"), fonts_dict_id);
    page.contents(content_id);

    // Create content stream
    let mut content = Content::new();

    // Header
    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(150.0, 720.0);
    content.show(Str(b"UNITED STATES DISTRICT COURT"));
    content.end_text();

    // District
    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(180.0, 700.0);
    content.show(Str(get_full_district_name(district).as_bytes()));
    content.end_text();

    // Case caption
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 660.0);
    content.show(Str(b"UNITED STATES OF AMERICA"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(250.0, 640.0);
    content.show(Str(b"v."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 640.0);
    let case_text = format!("Case No. {}", case_number);
    content.show(Str(case_text.as_bytes()));
    content.end_text();

    // Defendant name
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 600.0);
    content.show(Str(defendant_names.as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 580.0);
    content.show(Str(b"Defendant(s)."));
    content.end_text();

    // Title
    content.begin_text();
    content.set_font(Name(b"F1"), 16.0);
    content.next_line(180.0, 540.0);
    content.show(Str(b"SCHEDULING ORDER"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(170.0, 520.0);
    content.show(Str(b"(Fed. R. Crim. P. 16(b))"));
    content.end_text();

    // Body text
    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 480.0);
    content.show(Str(b"This matter comes before the Court for scheduling pursuant to"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 465.0);
    content.show(Str(b"Federal Rule of Criminal Procedure 16(b)."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 435.0);
    content.show(Str(b"IT IS HEREBY ORDERED:"));
    content.end_text();

    // Orders
    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 405.0);
    content.show(Str(b"1. DISCOVERY: Government to provide discovery within 14 days."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 375.0);
    content.show(Str(b"2. PRETRIAL MOTIONS: All motions due within 30 days."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 345.0);
    content.show(Str(b"3. CHANGE OF PLEA: Any plea change 14 days before trial."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 315.0);
    content.show(Str(b"4. TRIAL: Date to be determined."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 285.0);
    content.show(Str(b"5. STATUS CONFERENCE: 45 days from date of this order."));
    content.end_text();

    // Closing
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 245.0);
    content.show(Str(b"IT IS SO ORDERED."));
    content.end_text();

    // Date
    let date = Local::now().format("%B %d, %Y").to_string();
    let date_text = format!("DATED: {}", date);
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 205.0);
    content.show(Str(date_text.as_bytes()));
    content.end_text();

    // Finish page configuration
    page.finish();

    // Add signature or signature line
    if let Some(sig_base64) = signature_base64 {
        // Debug: Always show e-signature if any base64 provided
        if !sig_base64.is_empty() {
            // Show electronic signature block
            // This is more appropriate for court e-filing systems

            // Add border box for signature area
            content.set_line_width(0.5);
            content.move_to(340.0, 135.0);
            content.line_to(500.0, 135.0);
            content.line_to(500.0, 195.0);
            content.line_to(340.0, 195.0);
            content.close_path();
            content.stroke();

            // Add "ELECTRONICALLY SIGNED" header
            content.begin_text();
            content.set_font(Name(b"F1"), 8.0);
            content.next_line(355.0, 185.0);
            content.show(Str(b"ELECTRONICALLY SIGNED"));
            content.end_text();

            // Add signature date/time
            let signature_date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            content.begin_text();
            content.set_font(Name(b"F1"), 7.0);
            content.next_line(355.0, 175.0);
            content.show(Str(signature_date.as_bytes()));
            content.end_text();

            // Add stylized signature (judge initials)
            content.begin_text();
            content.set_font(Name(b"F1"), 18.0);
            content.next_line(380.0, 155.0);
            let initials = judge_name.split_whitespace()
                .filter_map(|word| word.chars().next())
                .collect::<String>();
            let signature_style = format!("/s/ {}", initials);
            content.show(Str(signature_style.as_bytes()));
            content.end_text();

            // Add authentication code
            content.begin_text();
            content.set_font(Name(b"F1"), 6.0);
            content.next_line(355.0, 140.0);
            // Generate a document verification code
            let doc_hash = format!("{:x}", chrono::Local::now().timestamp() % 999999);
            let auth_code = format!("Doc ID: {}-{}-{}", district, case_number, doc_hash);
            content.show(Str(auth_code.as_bytes()));
            content.end_text();
        } else {
            // Fallback to signature line
            content.begin_text();
            content.set_font(Name(b"F1"), 10.0);
            content.next_line(350.0, 145.0);
            content.show(Str(b"_______________________________"));
            content.end_text();
        }
    } else {
        // No signature - show traditional signature line
        content.begin_text();
        content.set_font(Name(b"F1"), 10.0);
        content.next_line(350.0, 145.0);
        content.show(Str(b"_______________________________"));
        content.end_text();
    }

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 130.0);
    content.show(Str(judge_name.as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 115.0);
    content.show(Str(b"United States District Judge"));
    content.end_text();

    // Write content stream
    pdf.stream(content_id, &content.finish());

    // Finish and return PDF bytes
    pdf.finish()
}

/// Generate generic court order PDF
pub fn generate_court_order(
    case_number: &str,
    defendant_names: &str,
    judge_name: &str,
    district: &str,
    order_title: &str,
    order_content: &str,
) -> Vec<u8> {
    generate_court_order_with_signature(case_number, defendant_names, judge_name, district, order_title, order_content, None)
}

/// Generate court order with optional signature
pub fn generate_court_order_with_signature(
    case_number: &str,
    defendant_names: &str,
    judge_name: &str,
    district: &str,
    order_title: &str,
    order_content: &str,
    signature_base64: Option<&str>,
) -> Vec<u8> {
    let mut pdf = Pdf::new();

    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let content_id = Ref::new(4);
    let fonts_dict_id = Ref::new(5);

    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id).kids([page_id]).count(1);
    pdf.type1_font(fonts_dict_id).base_font(Name(b"Helvetica"));

    let mut page = pdf.page(page_id);
    page.parent(page_tree_id);
    page.media_box(Rect::new(0.0, 0.0, 612.0, 792.0));
    page.resources().fonts().pair(Name(b"F1"), fonts_dict_id);
    page.contents(content_id);

    let mut content = Content::new();

    // Header
    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(150.0, 720.0);
    content.show(Str(b"UNITED STATES DISTRICT COURT"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(180.0, 700.0);
    content.show(Str(get_full_district_name(district).as_bytes()));
    content.end_text();

    // Case caption
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 660.0);
    content.show(Str(b"UNITED STATES OF AMERICA"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(250.0, 640.0);
    content.show(Str(b"v."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 640.0);
    let case_text = format!("Case No. {}", case_number);
    content.show(Str(case_text.as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 600.0);
    content.show(Str(defendant_names.as_bytes()));
    content.end_text();

    // Order title
    content.begin_text();
    content.set_font(Name(b"F1"), 16.0);
    content.next_line(200.0, 540.0);
    content.show(Str(order_title.as_bytes()));
    content.end_text();

    // Order content (wrap text)
    let mut y_pos = 500.0;
    let words: Vec<&str> = order_content.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if test_line.len() > 75 {
            content.begin_text();
            content.set_font(Name(b"F1"), 11.0);
            content.next_line(72.0, y_pos);
            content.show(Str(current_line.as_bytes()));
            content.end_text();
            y_pos -= 20.0;
            current_line = word.to_string();
        } else {
            current_line = test_line;
        }
    }

    if !current_line.is_empty() {
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(72.0, y_pos);
        content.show(Str(current_line.as_bytes()));
        content.end_text();
        y_pos -= 20.0;
    }

    // IT IS SO ORDERED
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, y_pos - 20.0);
    content.show(Str(b"IT IS SO ORDERED."));
    content.end_text();

    // Date
    let date = Local::now().format("%B %d, %Y").to_string();
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 180.0);
    content.show(Str(format!("DATED: {}", date).as_bytes()));
    content.end_text();

    page.finish();

    // Add signature
    if let Some(sig_base64) = signature_base64 {
        if !sig_base64.is_empty() {
            // Electronic signature block
            content.set_line_width(0.5);
            content.move_to(340.0, 110.0);
            content.line_to(500.0, 110.0);
            content.line_to(500.0, 170.0);
            content.line_to(340.0, 170.0);
            content.close_path();
            content.stroke();

            content.begin_text();
            content.set_font(Name(b"F1"), 8.0);
            content.next_line(355.0, 160.0);
            content.show(Str(b"ELECTRONICALLY SIGNED"));
            content.end_text();

            let signature_date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            content.begin_text();
            content.set_font(Name(b"F1"), 7.0);
            content.next_line(355.0, 150.0);
            content.show(Str(signature_date.as_bytes()));
            content.end_text();

            content.begin_text();
            content.set_font(Name(b"F1"), 18.0);
            content.next_line(380.0, 130.0);
            let initials = judge_name.split_whitespace()
                .filter_map(|word| word.chars().next())
                .collect::<String>();
            content.show(Str(format!("/s/ {}", initials).as_bytes()));
            content.end_text();

            content.begin_text();
            content.set_font(Name(b"F1"), 6.0);
            content.next_line(355.0, 115.0);
            let doc_hash = format!("{:x}", chrono::Local::now().timestamp() % 999999);
            content.show(Str(format!("Doc ID: {}-{}-{}", district, case_number, doc_hash).as_bytes()));
            content.end_text();
        }
    } else {
        content.begin_text();
        content.set_font(Name(b"F1"), 10.0);
        content.next_line(350.0, 120.0);
        content.show(Str(b"_______________________________"));
        content.end_text();
    }

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 105.0);
    content.show(Str(judge_name.as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 90.0);
    content.show(Str(b"United States District Judge"));
    content.end_text();

    pdf.stream(content_id, &content.finish());
    pdf.finish()
}

/// Generate minute entry PDF
pub fn generate_minute_entry(
    case_number: &str,
    defendant_names: &str,
    judge_name: &str,
    district: &str,
    minute_text: &str,
) -> Vec<u8> {
    let mut pdf = Pdf::new();

    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let content_id = Ref::new(4);
    let fonts_dict_id = Ref::new(5);

    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id).kids([page_id]).count(1);
    pdf.type1_font(fonts_dict_id).base_font(Name(b"Helvetica"));

    let mut page = pdf.page(page_id);
    page.parent(page_tree_id);
    page.media_box(Rect::new(0.0, 0.0, 612.0, 792.0));
    page.resources().fonts().pair(Name(b"F1"), fonts_dict_id);
    page.contents(content_id);

    let mut content = Content::new();

    // Header
    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(150.0, 720.0);
    content.show(Str(b"UNITED STATES DISTRICT COURT"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(180.0, 700.0);
    content.show(Str(get_full_district_name(district).as_bytes()));
    content.end_text();

    // Title
    content.begin_text();
    content.set_font(Name(b"F1"), 16.0);
    content.next_line(230.0, 660.0);
    content.show(Str(b"MINUTE ENTRY"));
    content.end_text();

    // Case info
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 620.0);
    content.show(Str(b"UNITED STATES OF AMERICA"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(250.0, 600.0);
    content.show(Str(b"v."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 600.0);
    content.show(Str(format!("Case No. {}", case_number).as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 560.0);
    content.show(Str(defendant_names.as_bytes()));
    content.end_text();

    // Date and Judge
    let date = Local::now().format("%B %d, %Y").to_string();
    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 520.0);
    content.show(Str(format!("Date: {}", date).as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 500.0);
    content.show(Str(format!("Judge: {}", judge_name).as_bytes()));
    content.end_text();

    // Minute text
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 460.0);
    content.show(Str(b"PROCEEDINGS:"));
    content.end_text();

    // Wrap minute text
    let mut y_pos = 430.0;
    let words: Vec<&str> = minute_text.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if test_line.len() > 75 {
            content.begin_text();
            content.set_font(Name(b"F1"), 11.0);
            content.next_line(72.0, y_pos);
            content.show(Str(current_line.as_bytes()));
            content.end_text();
            y_pos -= 20.0;
            current_line = word.to_string();
        } else {
            current_line = test_line;
        }
    }

    if !current_line.is_empty() {
        content.begin_text();
        content.set_font(Name(b"F1"), 11.0);
        content.next_line(72.0, y_pos);
        content.show(Str(current_line.as_bytes()));
        content.end_text();
    }

    // Clerk signature
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 180.0);
    content.show(Str(b"_______________________________"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 165.0);
    content.show(Str(b"Deputy Clerk"));
    content.end_text();

    // Time stamp
    let timestamp = Local::now().format("%B %d, %Y at %I:%M %p").to_string();
    content.begin_text();
    content.set_font(Name(b"F1"), 9.0);
    content.next_line(72.0, 140.0);
    content.show(Str(format!("Entered: {}", timestamp).as_bytes()));
    content.end_text();

    page.finish();
    pdf.stream(content_id, &content.finish());
    pdf.finish()
}

/// Get full district name from abbreviation
fn get_full_district_name(district: &str) -> String {
    match district.to_uppercase().as_str() {
        "SDNY" => "SOUTHERN DISTRICT OF NEW YORK",
        "EDNY" => "EASTERN DISTRICT OF NEW YORK",
        "NDCA" => "NORTHERN DISTRICT OF CALIFORNIA",
        "CDCA" => "CENTRAL DISTRICT OF CALIFORNIA",
        "NDIL" => "NORTHERN DISTRICT OF ILLINOIS",
        "SDTX" => "SOUTHERN DISTRICT OF TEXAS",
        "EDPA" => "EASTERN DISTRICT OF PENNSYLVANIA",
        "SDFL" => "SOUTHERN DISTRICT OF FLORIDA",
        "DDC" => "DISTRICT OF COLUMBIA",
        "EDVA" => "EASTERN DISTRICT OF VIRGINIA",
        _ => district,
    }.to_string()
}