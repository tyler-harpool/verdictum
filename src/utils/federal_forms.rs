//! Federal court forms generation module
//! Implements common federal court forms from uscourts.gov

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
use chrono::Local;

/// Form AO 455: Waiver of an Indictment
pub fn generate_waiver_of_indictment(
    case_number: &str,
    defendant_name: &str,
    district: &str,
    charges: &str,
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
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(150.0, 740.0);
    content.show(Str(b"UNITED STATES DISTRICT COURT"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(180.0, 720.0);
    content.show(Str(get_full_district(district).as_bytes()));
    content.end_text();

    // Title
    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(200.0, 680.0);
    content.show(Str(b"WAIVER OF AN INDICTMENT"));
    content.end_text();

    // Case info
    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 640.0);
    content.show(Str(b"UNITED STATES OF AMERICA"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(250.0, 620.0);
    content.show(Str(b"v."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(350.0, 620.0);
    content.show(Str(format!("Case No. {}", case_number).as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 580.0);
    content.show(Str(defendant_name.as_bytes()));
    content.end_text();

    // Waiver text
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 540.0);
    content.show(Str(b"I understand that I have been accused of one or more offenses"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 525.0);
    content.show(Str(b"punishable by imprisonment for more than one year."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 495.0);
    content.show(Str(b"I was advised of the charges against me, as follows:"));
    content.end_text();

    // Charges
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 465.0);
    content.show(Str(charges.as_bytes()));
    content.end_text();

    // Rights waiver
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 425.0);
    content.show(Str(b"I was advised of my rights and the nature of the proposed"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 410.0);
    content.show(Str(b"charges to be filed or pending against me."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 380.0);
    content.show(Str(b"I waive my right to prosecution by indictment and consent"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 365.0);
    content.show(Str(b"to the filing of an information."));
    content.end_text();

    // Signature lines
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 280.0);
    content.show(Str(b"________________________________"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 265.0);
    content.show(Str(b"Defendant"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 280.0);
    content.show(Str(b"________________________________"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 265.0);
    content.show(Str(b"Date"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 220.0);
    content.show(Str(b"________________________________"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 205.0);
    content.show(Str(b"Attorney for Defendant"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 220.0);
    content.show(Str(b"________________________________"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 205.0);
    content.show(Str(b"Date"));
    content.end_text();

    page.finish();
    pdf.stream(content_id, &content.finish());
    pdf.finish()
}

/// Form AO 199A: Order Setting Conditions of Release
pub fn generate_conditions_of_release(
    case_number: &str,
    defendant_name: &str,
    district: &str,
    judge_name: &str,
    conditions: Vec<&str>,
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
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(150.0, 740.0);
    content.show(Str(b"UNITED STATES DISTRICT COURT"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(180.0, 720.0);
    content.show(Str(get_full_district(district).as_bytes()));
    content.end_text();

    // Title
    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(160.0, 680.0);
    content.show(Str(b"ORDER SETTING CONDITIONS OF RELEASE"));
    content.end_text();

    // Case info
    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 640.0);
    content.show(Str(b"UNITED STATES OF AMERICA"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(250.0, 620.0);
    content.show(Str(b"v."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(350.0, 620.0);
    content.show(Str(format!("Case No. {}", case_number).as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 580.0);
    content.show(Str(defendant_name.as_bytes()));
    content.end_text();

    // Order text
    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 540.0);
    content.show(Str(b"IT IS ORDERED that the defendant's release is subject to"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 525.0);
    content.show(Str(b"the following conditions:"));
    content.end_text();

    // Conditions
    let mut y_pos = 495.0;
    for (i, condition) in conditions.iter().enumerate() {
        content.begin_text();
        content.set_font(Name(b"F1"), 10.0);
        content.next_line(85.0, y_pos);
        content.show(Str(format!("{}. {}", i + 1, condition).as_bytes()));
        content.end_text();
        y_pos -= 20.0;
    }

    // Judge signature
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 180.0);
    content.show(Str(b"_______________________________"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 165.0);
    content.show(Str(judge_name.as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 150.0);
    content.show(Str(b"United States District Judge"));
    content.end_text();

    // Date
    let date = Local::now().format("%B %d, %Y").to_string();
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 120.0);
    content.show(Str(format!("Date: {}", date).as_bytes()));
    content.end_text();

    page.finish();
    pdf.stream(content_id, &content.finish());
    pdf.finish()
}

/// Form AO 245B: Judgment in a Criminal Case
pub fn generate_criminal_judgment(
    case_number: &str,
    defendant_name: &str,
    district: &str,
    judge_name: &str,
    plea: &str,
    counts: &str,
    sentence: &str,
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
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(150.0, 740.0);
    content.show(Str(b"UNITED STATES DISTRICT COURT"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(180.0, 720.0);
    content.show(Str(get_full_district(district).as_bytes()));
    content.end_text();

    // Title
    content.begin_text();
    content.set_font(Name(b"F1"), 14.0);
    content.next_line(180.0, 680.0);
    content.show(Str(b"JUDGMENT IN A CRIMINAL CASE"));
    content.end_text();

    // Case info
    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 640.0);
    content.show(Str(b"UNITED STATES OF AMERICA"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(250.0, 620.0);
    content.show(Str(b"v."));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(350.0, 620.0);
    content.show(Str(format!("Case No. {}", case_number).as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(72.0, 580.0);
    content.show(Str(defendant_name.as_bytes()));
    content.end_text();

    // Plea
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 540.0);
    content.show(Str(format!("The defendant entered a plea of: {}", plea).as_bytes()));
    content.end_text();

    // Counts
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 510.0);
    content.show(Str(format!("The defendant is adjudicated guilty of: {}", counts).as_bytes()));
    content.end_text();

    // Sentence header
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 470.0);
    content.show(Str(b"IMPRISONMENT"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 445.0);
    content.show(Str(format!("The defendant is hereby committed to the custody of the").as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 430.0);
    content.show(Str(b"Federal Bureau of Prisons for:"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 11.0);
    content.next_line(85.0, 405.0);
    content.show(Str(sentence.as_bytes()));
    content.end_text();

    // Supervised release
    content.begin_text();
    content.set_font(Name(b"F1"), 12.0);
    content.next_line(72.0, 365.0);
    content.show(Str(b"SUPERVISED RELEASE"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 340.0);
    content.show(Str(b"Upon release from imprisonment, the defendant shall be on"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 325.0);
    content.show(Str(b"supervised release for a term of: 3 years"));
    content.end_text();

    // Judge signature
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 180.0);
    content.show(Str(b"_______________________________"));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 165.0);
    content.show(Str(judge_name.as_bytes()));
    content.end_text();

    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(350.0, 150.0);
    content.show(Str(b"United States District Judge"));
    content.end_text();

    // Date
    let date = Local::now().format("%B %d, %Y").to_string();
    content.begin_text();
    content.set_font(Name(b"F1"), 10.0);
    content.next_line(72.0, 120.0);
    content.show(Str(format!("Date: {}", date).as_bytes()));
    content.end_text();

    page.finish();
    pdf.stream(content_id, &content.finish());
    pdf.finish()
}

/// Helper function to get full district name
fn get_full_district(district: &str) -> String {
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