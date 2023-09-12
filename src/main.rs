use reqwest::{header, Client};
use serde_json::{json, Value};
use serde::Deserialize;
use clap::{App, Arg};
use std::env;
use std::error::Error;

// Azure DevOps Personal Access Token (PAT)
const AZURE_DEVOPS_PAT_ENV: &str = "AZURE_DEVOPS_EXT_PAT";

#[derive(Debug)]
pub struct Config {
    pub organization: String,
    pub project: String,
    pub pipeline_id: u32,
    pub template_parameters: String,
}

#[derive(Debug, Deserialize)]
struct Response {
    pipeline: PipelineInfo,
}

#[derive(Debug, Deserialize)]
struct PipelineInfo {
    //url: String,
    //id: i32,
    //revision: i32,
    name: String,
    //folder: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = get_args().unwrap();

    // Check for the Azure DevOps PAT environment variable
    let pat = match env::var(AZURE_DEVOPS_PAT_ENV) {
        Ok(pat) => pat,
        Err(_) => {
            eprintln!(
                "Please set the {} environment variable with your Azure DevOps Personal Access Token.",
                AZURE_DEVOPS_PAT_ENV
            );
            std::process::exit(1);
        }
    };

    // Create an HTTP client
    let client = Client::new();

    // create a valid json body from the template parameters
    let template_params = &config.template_parameters;
    let json_result = match pipeline_parameters(template_params) {
        Ok(json_result) => json_result,
        Err(e) => panic!("failed json parsing: {}", e),
    };


    // Send a POST request to trigger a pipeline run
    let response = client
        .post(&pipeline_run_url(config))
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Basic {}", base64::encode(&format!(":{}", pat))))
        .json(&json_result)
        .send()
        .await?;

    // Check the response status code
    match response.status() {
        reqwest::StatusCode::OK => {
            let body = response.bytes().await?;
            let response_str = String::from_utf8_lossy(&body);
            let response_object: Response = serde_json::from_str(&response_str)
                .unwrap();

            println!("Pipeline [{}] triggered successfully!",  
                response_object.pipeline.name);
            Ok(())
        }
        _ => {
            eprintln!("Failed to trigger the pipeline run: {:?}", 
                response.status());
            std::process::exit(1);
        }
    }
}

// Pipeline run URL builder function
fn pipeline_run_url(config :Config) -> String {
    format!(
        "https://dev.azure.com/{}/{}/_apis/pipelines/{}/runs?api-version=7.1-preview.1",
        config.organization, config.project, config.pipeline_id
    )
}

fn is_valid_u32(value: String) -> Result<(), String> {
    match value.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Invalid u32 value")),
    }
}

fn pipeline_parameters(template_params: &str) -> Result<Value, Box<dyn Error>> {
    // Get the parameters as a string
    let params_str = template_params;

    // Parse the JSON string into a serde_json::Value
    let parsed_json_result: Result<Value, _> = serde_json::from_str(params_str);

    match parsed_json_result {
        Ok(json_obj) => {
            // Ensure the JSON object is a JSON object (not an array, null, etc.)
            if let Value::Object(template_parameters) = json_obj {
                // Prepare the JSON request body with template parameters
                let request_body = json!({
                    "resources": {
                        "repositories": {
                            "self": {},
                        },
                    },
                    "templateParameters": template_parameters,
                });

                // Returns the generated JSON for testing
                Ok(request_body)
            } else {
                panic!("Invalid JSON object.");
            }
        }
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            Err(Box::new(e))
        }
    }
}

pub fn get_args() -> Result<Config, Box<dyn Error>> {
    // Define and parse command-line arguments using clap
    let matches = App::new("azure_pipelines_runs")
        .arg(
            Arg::with_name("organization")
                .short("o")
                .long("organization")
                .required(true)
                .takes_value(true)
                .help("Azure DevOps Organization name"),
        )
        .arg(
            Arg::with_name("project")
                .short("p")
                .long("project")
                .required(true)
                .takes_value(true)
                .help("Azure DevOps Project"),
        )
        .arg(
            Arg::with_name("pipeline_id")
                .short("id")
                .long("pipeline_id")
                .required(true)
                .takes_value(true)
                .help("Azure Pipeline ID")
                .validator(is_valid_u32),
        )
        .arg(
            Arg::with_name("template_parameters")
                .short("t")
                .long("template_parameters")
                .required(false)
                .takes_value(true)
                .help("Pipeline Template Parameters"),
        )
        .get_matches();

    Ok(Config {
        organization: matches.value_of("organization")
                             .expect("organization is required")
                             .to_string(),
        project: matches.value_of("project")
                             .expect("project is required")
                             .to_string(),
        pipeline_id: matches.value_of("pipeline_id")
                             .expect("pipeline_id is required")
                             .parse::<u32>()?,
        template_parameters: matches.value_of("template_parameters")
                                    .unwrap_or("")
                                    .to_string(),
    })
}