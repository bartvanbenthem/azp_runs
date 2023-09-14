# Azure Pipelines RUNS Client
cli client to execute Azure pipelines over the Runs REST API with personal acces token authentication. Integrates with Azure pipeline template parameters for dynamic execution. When specified, the pipeline run state is being tracked automatically and the status results are being updated for every run. Perfect for the creation and integration of orchestrators that rely on complex and imperative orchestration on multiple other pipelines for end-to-end process automation.

## Usage
```text
azure_pipelines_runs 

USAGE:
    azp_runs [FLAGS] [OPTIONS] --organization <organization> --pipeline_id <pipeline_id> --project <project>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -w, --watch      Watch pipeline status and block untill finished

OPTIONS:
    -o, --organization <organization>                  Azure DevOps Organization name
    -i, --pipeline_id <pipeline_id>                    Azure Pipeline ID
    -p, --project <project>                            Azure DevOps Project
    -t, --template_parameters <template_parameters>    Pipeline Template Parameters
```

## Set environment variables
```bash
#!/bin/bash
export AZURE_DEVOPS_EXT_PAT='Azure DevOps personal access token'
```

## Build and Run
```bash
#!/bin/bash

git clone https://github.com/bartvanbenthem/azp_runs.git
cd azp_runs

# build
cargo build --release

# execute an Azure pipeline
./target/release/azp_runs -o "OrganizationName" -p "ProjectName" -i 999

# execute an Azure pipeline with input parameters and wait for completion
./target/release/azp_runs -o "OrganizationName" -p "ProjectName" -i 999 \
    --template_parameters "{\"param1\": \"value1\", \"param2\": \"value2\"}" \
    --watch

# the --watch parameter can be used in more complex orchestration scenarios,
# that requires different parts of the pipeline to wait on the result and
# block further execution untill the pipeline status is completed
# or just to give more insight into the pipeline status and final result.
```