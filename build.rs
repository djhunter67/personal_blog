use std::{env, fs};

/// This will change the file multiple times per build.
/// This build.rs file will work in tandem with the commit
/// and push hooks to change the secret being injected here.
const MONGO_LOCATION: &str = "10.20.20.205:27017/admin?replicaSet=rs0";
const REDIS_LOCATION: &str = "10.20.20.202:6379";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dest_path = String::from("./settings/base.yaml");

    // Load the .env file to build the scripts environment
    dotenvy::dotenv().expect("Failed to load the .env file");

    // Retrieve the specific secret
    let mongo_val = env::var("DATABASE_PW").expect("Unable to find or parse the MONGO PW");
    let redis_val = env::var("REDIS_PW").expect("Unable to find or parse the REDIS PW");

    // Define the output path in the build directory
    // let _out_dir = env::var("OUT_DIR").expect("The OUT_DIR is unknown");

    // Create the yaml content
    let mongo_uri = format!(" uri: \"mongodb://admin:{mongo_val}@{MONGO_LOCATION}\"");
    let redis_connect = format!("  uri: \"redis://:{redis_val}@{REDIS_LOCATION}\"");

    // Capture the contents of the yaml file
    // let _ = File::read_to_string(
    //     &mut File::open(&dest_path).expect("File not found"),
    //     &mut prev_contents,
    // )
    // .expect("Unable to convert the contents of the file");

    let prev_contents = fs::read_to_string(&dest_path)?;

    let mut mongo_content: Vec<&str> = Vec::with_capacity(prev_contents.len());
    // let mut redis_content: Vec<&str> = Vec::with_capacity(prev_contents.len());
    let mut uri_count: u8 = 0;

    // If more secrets add the replacement here
    for line in prev_contents.lines() {
        if line.contains("uri:") {
            uri_count += 1;
            if line.contains("<password>") && !uri_count.gt(&1) {
                mongo_content.push(&mongo_uri);
                continue;
            }
            if line.contains("redis") && uri_count.gt(&1) {
                mongo_content.push(&redis_connect);
                continue;
            }
        }

        mongo_content.push(line);
    }

    let mongo_content: String = mongo_content.join("\n");
    // let redis_content: String = redis_content.join("\n");

    // Write to the base.yaml file
    fs::write(&dest_path, mongo_content)?;
    // fs::write(&dest_path, redis_content)?;

    // Rerun the build if the .env file changes
    println!("cargo:rerun-if-changed={dest_path}");

    Ok(())
}
