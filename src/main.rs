use clap::{App, Arg};
use reqwest::{header, Client};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;

// Azure DevOps Personal Access Token (PAT)
const AZURE_DEVOPS_PAT_ENV: &str = "AZURE_DEVOPS_EXT_PAT";

#[derive(Debug)]
pub struct Config {
    pub organization: String,
    pub project: String,
    pub pipeline_id: u32,
    pub template_parameters: String,
    pub watch: bool,
}

#[derive(Debug, Deserialize)]
struct Response {
    pipeline: PipelineInfo,
    id: u32,
    state: String,
}

#[derive(Debug, Deserialize)]
struct PipelineInfo {
    //url: String,
    id: i32,
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

    // create a valid json body from the template parameters
    let json_result;
    if config.template_parameters.len() != 0 {
        let template_params = &config.template_parameters;
        json_result = match pipeline_parameters(template_params) {
            Ok(json_result) => json_result,
            Err(e) => panic!("failed json parsing: {}", e),
        };
    } else {
        json_result = Value::Object(Map::new());
    }

    // Create an HTTP client
    let client = Client::new();
    // Send a POST request to trigger a pipeline run
    let response = client
        .post(&pipeline_run_url(&config))
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::AUTHORIZATION,
            format!("Basic {}", base64::encode(&format!(":{}", pat))),
        )
        .json(&json_result)
        .send()
        .await?;

    // Check the response status code
    match response.status() {
        reqwest::StatusCode::OK => {
            let body = response.bytes().await?;
            let response_str = String::from_utf8_lossy(&body);
            let response_object: Response = serde_json::from_str(&response_str).unwrap();

            println!(
                "Pipeline [{}] with id [{}] triggered successfully, run id = [{}]",
                response_object.pipeline.name, response_object.pipeline.id, response_object.id
            );

            if config.watch == true {
                // Call the watch function asynchronously
                let watch_result =
                    pipeline_watch(client, &config, pat.clone(), response_object.id).await;

                // Handle the result of the watch function
                match watch_result {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Error in watch function: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
        _ => {
            eprintln!(
                "Failed to trigger the pipeline run: {:?}",
                response.status()
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

fn is_valid_u32(value: String) -> Result<(), String> {
    match value.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Invalid u32 value")),
    }
}

fn pipeline_parameters(template_params: &str) -> Result<Value, Box<dyn Error>> {
    // Parse the JSON string into a serde_json::Value
    let parsed_json_result = serde_json::from_str(template_params);

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
                .short("i")
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
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .required(false)
                .takes_value(false)
                .help("Watch pipeline status and block untill finished"),
        )
        .get_matches();

    Ok(Config {
        organization: matches
            .value_of("organization")
            .expect("organization is required")
            .to_string(),
        project: matches
            .value_of("project")
            .expect("project is required")
            .to_string(),
        pipeline_id: matches
            .value_of("pipeline_id")
            .expect("pipeline_id is required")
            .parse::<u32>()?,
        template_parameters: matches
            .value_of("template_parameters")
            .unwrap_or("")
            .to_string(),
        watch: matches.is_present("watch"),
    })
}

// Pipeline run URL builder function
fn pipeline_run_url(config: &Config) -> String {
    format!(
        "https://dev.azure.com/{}/{}/_apis/pipelines/{}/runs?api-version=7.1-preview.1",
        config.organization, config.project, config.pipeline_id
    )
}

async fn pipeline_watch(
    client: Client,
    config: &Config,
    pat: String,
    run_id: u32,
) -> Result<(), Box<dyn Error>> {
    let pipeline_status_url = format!(
        "https://dev.azure.com/{}/{}/_apis/pipelines/{}/runs/{}?api-version=7.1-preview.1",
        config.organization, config.project, config.pipeline_id, run_id
    );

    loop {
        // Send a GET request to the Azure DevOps API to get the pipeline run status
        let response = client
            .get(&pipeline_status_url)
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json")
            .header(
                header::AUTHORIZATION,
                format!("Basic {}", base64::encode(&format!(":{}", pat))),
            )
            .send()
            .await?;

        // Check if the request was successful
        if response.status().is_success() {
            let status_json: Response = response.json().await?;
            let status = status_json.state.as_str();
            // Check if the pipeline has finished executing
            if status == "completed" || status == "canceled" || status == "failed" {
                println!("Pipeline has finished with status: {}", status);
                break; // Exit the loop
            } else {
                println!("Pipeline status: {}", status);
            }
        } else {
            eprintln!(
                "Failed to retrieve pipeline status: {:?}",
                response.status()
            );
        }
        // Sleep for a while before checking again (e.g., every 30 seconds)
        thread::sleep(Duration::from_secs(10));
    }
    Ok(())
}
