use eframe::egui;
use reqwest;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use std::thread;

use dotenv::dotenv;
use std::env;
use tokio::spawn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("SpaceObjectTracking",native_options,Box::new(|cc| Ok(Box::new(SpaceObjectTracking::new(cc)))));

    Ok(())
}

#[derive(Default,Clone)]
struct SpaceObjectTracking {
    tracked_objects : Vec<SpaceObject>,
    transaction_count :u64,
}

#[derive(Debug,Clone)]
pub struct SpaceObject{
    pub satid:String,
    pub satname:String,
    pub intDesignator: String,
    pub launchDate: String,
    pub satlat: f64,
    pub satlng: f64,
    pub satalt: f64,
    pub tle: Option<String>,
}

impl SpaceObjectTracking {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut tracked_objects: Vec<SpaceObject> = Vec::new();

        let mocked_object = SpaceObject {
            satid: "0".to_string(),
            satname: "Dummy".to_string(),
            intDesignator: "Designator".to_string(),
            launchDate: "ReleasedToday".to_string(),
            satlat: 0.,
            satlng: 0.,
            satalt: 0.,
            tle: None,
        };
       
        tracked_objects.push(mocked_object.clone());
        tracked_objects.push(mocked_object.clone());
        tracked_objects.push(mocked_object.clone());

        Self {
            tracked_objects,
            transaction_count: 0,
        }
    }


    fn get_mocked_objects() -> Vec<SpaceObject> {
        let mut tracked_objects: Vec<SpaceObject> = Vec::new();

        let mocked_object = SpaceObject {
            satid: "0".to_string(),
            satname: "Dummy".to_string(),
            intDesignator: "Designator".to_string(),
            launchDate: "ReleasedToday".to_string(),
            satlat: 0.,
            satlng: 0.,
            satalt: 0.,
            tle: None,
        };
       
        tracked_objects.push(mocked_object.clone());
        tracked_objects.push(mocked_object.clone());
        tracked_objects.push(mocked_object.clone());
        
        tracked_objects

    }
// Box<dyn Error>: A heap allocated dynamic error Type that represents any Error
// implementing trait Error

    async fn update_tracked_objects(&mut self) -> Result<Vec<SpaceObject>, Box<dyn Error + Send>> {
        dotenv().ok();
        let api_key = env::var("N2YO_API_KEY").expect("N2YO_API_KEY not set");
        let latitude = env::var("LATITUDE").expect("LATITUDE not set");
        let longitude = env::var("LONGITUDE").expect("LONGITUDE not set");

        let api_key_str : String = api_key.parse().expect("Not able to parse API_KEY.");
        let latitude_str : String = latitude.parse().expect("Not able to parse Latitude.");
        let longitude_str : String = longitude.parse().expect("Not able to parse Longitude.");

        // n2yo Example
        // This example is for retrieve Space Station (25544) positions for next 1 seconds. 
        let mut api_endpoint = String::new();
        api_endpoint.push_str("https://api.n2yo.com/rest/v1/satellite/above/");
        api_endpoint.push_str(&latitude_str);
        api_endpoint.push_str("/");
        api_endpoint.push_str(&longitude_str);
        api_endpoint.push_str("/0/70/32/&apiKey=");
        api_endpoint.push_str(&api_key);
        let response = reqwest::get(api_endpoint)
            .await
            .map_err(|err| Box::new(err) as Box<dyn Error + Send>)?
            .text()
            .await
            .map_err(|err| Box::new(err) as Box<dyn Error + Send>)?;

        let parsed_response: Value = match serde_json::from_str(&response) {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!("Failed to parse JSON: {}", err);
                return Err(Box::new(err) as Box<dyn Error + Send>);
            } 
        };
        
        let mut tracked_objects: Vec<SpaceObject> = Vec::new();

        for space_object in parsed_response["above"].as_array().unwrap(){
            tracked_objects.push(SpaceObject{
            satid: space_object["satid"].to_string(),
            satname: space_object["satname"].to_string(),
            intDesignator: space_object["intDesignator"].to_string(),
            launchDate: space_object["launchDate"].to_string(),
            satlat: space_object["satlat"].as_f64().expect("Error when converting Latitude"),
            satlng: space_object["satlng"].as_f64().expect("Error when converting Longitude"),
            satalt: space_object["satalt"].as_f64().expect("Error when converting Altitude"),
            tle: None,
            });
        }

        Ok(tracked_objects)
    }
}

// Where things are updated on the UI Screen
impl eframe::App for SpaceObjectTracking {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       egui::CentralPanel::default().show(ctx, |ui| {
           let mut update_ui = true;
           if ui.add(egui::Button::new("Update Tracked Objects List🔄")).clicked() {
                println!("Synchronize!");
                // Spawn a task to run the async function
                let mut self_clone = self.clone();

                let joinhandle = thread::spawn(move || {
                    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                    runtime.block_on(self_clone.update_tracked_objects())
                });

                // Use the join handle to wait for the thread and get its result
                match joinhandle.join() {
                    Ok(result) => match result {
                        Ok(data) => {
                            println!("Thread finished with result: {:?}", data);
                            self.tracked_objects = data; // Save the result to tracked_objects
                            update_ui = true;
                        }
                        Err(err) => println!("Error occurred: {}", err),
                    },
                    Err(e) => println!("Thread panicked: {:?}", e),
                }
           }
           if(update_ui){
                egui::ScrollArea::vertical().show(ui, |ui|{
                    ui.vertical_centered(|ui| {
                    for space_object in &self.tracked_objects{
                            ui.label(space_object.satname.clone());
                            ui.label(space_object.satlat.to_string());
                            ui.label(space_object.satlng.to_string());
                        } 
                    });
                });
                update_ui = false;
           } 
       });
   }
}
