use std::collections::HashSet;

use serde_json::Value;

#[derive(Copy, Clone)]
pub enum StudyLevel {
    Undergrad,
    Postgrad,
}

#[derive(Serialize, Deserialize)]
pub struct Course {
    title:                  String,
    credit_points:          String,
    is_general_education:   String,
    status:                 String,
    description:            String,
    educational_area:       String,
    academic_calendar_type: String,
    academic_org:           String,
    enrolment_requirements: String,
    offering_terms:         String,
    delivery_mode:          String,
    equivalent_courses:     Vec<String>,
    exclusion_courses:      Vec<String>,
    course_outline_url:     String,
}

pub fn get_course(code: &str, year: u32, study_level: StudyLevel) -> Option<Course> {
    // no response, no text => 404
    let resp = reqwest::blocking::get(
            &format!(
                "https://www.handbook.unsw.edu.au/api/content/render/false/query/+contentType:unsw_psubject%20+unsw_psubject.studyLevelURL:{}%20+unsw_psubject.implementationYear:{}%20+unsw_psubject.code:{}%20+deleted:false%20+working:true%20+live:true%20+languageId:1%20/orderBy/modDate%20desc",
                study_level.to_string(),
                year,
                code,
            )
        ).ok()?
        .text().ok()?;

    // invalid json response => 404
    let resp: Value = serde_json::from_str(&resp).ok()?;

    // no content => 404
    let contents = {
        let contentlets = resp.get("contentlets")?;
        contentlets.get(0)?
    };

    let data: Option<Value> = contents.get("data")
        .and_then(|data| data.as_str())
        .and_then(|data| serde_json::from_str(data).ok());
    
    let enrolment_rules = data.as_ref()
        .and_then(|data| data.get("enrolment_rules"))
        .and_then(|data| data.get(0));
    
    let hb_entires = data.as_ref()
        .and_then(|data| data.get("hb_entries"))
        .and_then(|data| data.get(0));

    let delivery_variations = data.as_ref()
        .and_then(|data| data.get("hb_delivery_variations"))
        .and_then(|data| data.get(0));

    let offering_detail = data.as_ref()
        .and_then(|data| data.get("offering_detail"));

    let equivalents = data.as_ref()
        .and_then(|data| data.get("eqivalents"))
        .and_then(|data| data.as_array())
        .unwrap_or(&Vec::new())
        .into_iter()
        .filter_map(|e| e.get("assoc_code"))
        .filter_map(|e| e.as_str())
        .map(|e| e.to_owned())
        .collect::<Vec<_>>();

    let exclusions = data.as_ref()
        .and_then(|data| data.get("exclusion"))
        .and_then(|data| data.as_array())
        .unwrap_or(&Vec::new())
        .into_iter()
        .filter_map(|e| e.get("assoc_code"))
        .filter_map(|e| e.as_str())
        .map(|e| e.to_owned())
        .collect::<Vec<_>>();

    Some(
        Course {
            title:                  contents.get("title")
                                        .and_then(|title| title.as_str())
                                        .map(|title| title.to_owned())
                                        .unwrap_or(String::new()),

            credit_points:          contents.get("creditPoints")
                                        .and_then(|cp| cp.as_str())
                                        .map(|cp| cp.to_owned())
                                        .unwrap_or(String::new()),

            is_general_education:   contents.get("generalEducation")
                                        .and_then(|ge| ge.as_str())
                                        .map(|ge| ge.to_owned())
                                        .unwrap_or(String::new()),
            
            status:                 contents.get("status")
                                        .and_then(|status| status.as_str())
                                        .map(|status| status.to_owned())
                                        .unwrap_or(String::new()),

            description:            contents.get("description")
                                        .and_then(|desc| desc.as_str())
                                        .map(|desc| desc.to_owned())
                                        .unwrap_or(String::new()),

            educational_area:       contents.get("educationalArea")
                                        .and_then(|ea| ea.as_str())
                                        .map(|ea| ea.to_owned())
                                        .unwrap_or(String::new()),

            academic_calendar_type: data.as_ref()
                                        .and_then(|data| data.get("academic_calendar_type"))
                                        .and_then(|act| act.get("value"))
                                        .and_then(|act| act.as_str())
                                        .map(|act| act.to_owned())
                                        .unwrap_or(String::new()),

            academic_org:           data.as_ref()
                                        .and_then(|data| data.get("academic_org"))
                                        .and_then(|ao| ao.get("value"))
                                        .and_then(|ao| ao.as_str())
                                        .map(|ao| ao.to_owned())
                                        .unwrap_or(String::new()),

            enrolment_requirements: enrolment_rules.and_then(|er| er.get("description"))
                                                   .and_then(|er| er.as_str())
                                                   .map(clean_html)
                                                   .unwrap_or(String::new()),
            
            offering_terms:         offering_detail.and_then(|od| od.get("offering_terms"))
                                                   .and_then(|ot| ot.as_str())
                                                   .map(|ot| ot.to_owned())
                                                   .unwrap_or(String::new()),

            delivery_mode:          delivery_variations.and_then(|dv| dv.get("delivery_mode"))
                                                       .and_then(|dm| dm.get("value"))
                                                       .and_then(|dm| dm.as_str())
                                                       .map(|dm| dm.to_owned())
                                                       .unwrap_or(String::new()),

            equivalent_courses:     equivalents,

            exclusion_courses:      exclusions,
            
            course_outline_url:     hb_entires.and_then(|hbe| hbe.get("link_url"))
                                              .and_then(|url| url.as_str())
                                              .map(|url| url.to_owned())
                                              .unwrap_or(String::new()),
        }
    )
}

fn clean_html(str: &str) -> String {
    ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(str)
        .to_string()
}

impl ToString for StudyLevel {
    
    fn to_string(&self) -> String {
        match self {
            Self::Undergrad => "undergraduate".to_owned(),
            Self::Postgrad  => "postgraduate".to_owned(),
        }
    }

}

impl From<&str> for StudyLevel {
    fn from(str: &str) -> Self {
        match &*str.to_ascii_lowercase() {
            "undergraduate" => Self::Undergrad,
            "postgraduate"  => Self::Postgrad,
            _               => panic!("Could not convert {} into StudyLevel", str),
        }
    }
}
