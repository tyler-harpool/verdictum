//! Simple PDF generation for court documents

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
use chrono::Local;

/// Generate a simple Rule 16(b) Scheduling Order PDF
pub fn generate_simple_rule_16b_order(
    case_number: &str,
    defendant_names: &str,
    judge_name: &str,
    district: &str,
) -> Vec<u8> {
    // Create a new PDF
    let mut pdf = Pdf::new();

    // Create catalog and page tree
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let _font_id = Ref::new(4);
    let content_id = Ref::new(5);

    // Create catalog
    pdf.catalog(catalog_id).pages(page_tree_id);

    // Create page tree
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    // Create page (8.5" x 11" in points: 612 x 792)
    let mut page = pdf.page(page_id);
    page.parent(page_tree_id);
    page.media_box(Rect::new(0.0, 0.0, 612.0, 792.0));

    // Add fonts dictionary
    let fonts_dict_id = Ref::new(6);
    page.resources().fonts().pair(Name(b"F1"), fonts_dict_id);

    page.contents(content_id);
    page.finish();

    // Add Helvetica font via separate dictionary
    pdf.type1_font(fonts_dict_id).base_font(Name(b"Helvetica"));

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

    // Signature line
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 145.0);
    content.show(Str(b"_______________________________"));
    content.end_text();

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