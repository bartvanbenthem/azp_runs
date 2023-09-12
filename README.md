# azp_runs
cli client to execute Azure pipelines over REST API with personal acces token authentication. when specified the pipeline run state is being tracked automatically and the result is being updated after every run. Integrates with Azure pipeline parameter specifications.

## Usage
```text
azure_pipelines_runs 

USAGE:
    azp_runs [OPTIONS] --organization <organization> --pipeline_id <pipeline_id> --project <project>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organization <organization>                  Azure DevOps Organization name
    -i, --pipeline_id <pipeline_id>                    Azure Pipeline ID
    -p, --project <project>                            Azure DevOps Project
    -t, --template_parameters <template_parameters>    Pipeline Template Parameters
```

## Set environment variables
```bash
# Bash
export AZURE_DEVOPS_EXT_PAT='Azure DevOps personal access token'
```

## Build and Run
```bash
cargo run --  --organization "Organization_Name" \
              --pipeline_id 999 \
              --project "Project_Name" \
              --template_parameters "{\"param1\": \"value1\", \"param2\": \"value2\"}"
```