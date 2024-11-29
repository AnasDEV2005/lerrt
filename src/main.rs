use cron::Schedule;
use dialoguer::Select;
use notify_rust::Notification;
use simple_db::SimpleDB;
use std::fs;
use std::io::stdin;
use std::str::FromStr;
use std::thread;
use std::time::Duration;





//===============================================================================
fn main() {
    run_menu();
}

//===============================================================================







// HACK: idk what i cooked bro
fn fix_timezone(time_change: i32, vec: Vec<(u32, String)>) -> Vec<(u32, String)> {
    let mut vecc = vec;
    for (time, _activity) in vecc.iter_mut() {
        let mut new_time = *time as i32 + time_change;
        if new_time < 0 {
            new_time = 24 + new_time;
        } else {
            new_time = new_time ;
        }
        *time = new_time as u32;
    }
    vecc
}







fn send_notification(message: &str) {
    let notification = Notification::new()
        .summary(message)
        .body(message)
        .icon("dialog-information")
        .show()
        .unwrap();
    notification.wait_for_action(|action| {
        println!("Notification action: {:?}", action);
    });
}




fn schedule_notifications(schedule_expr: &str, message: &str) {
    println!("Scheduled notif for: {}", message);
    let schedule = Schedule::from_str(&schedule_expr).unwrap();
    loop {
        let now = chrono::Utc::now();
        let next_occurrence = schedule.upcoming(chrono::Utc).next().unwrap();
        let duration = next_occurrence.signed_duration_since(now);
        if duration.num_seconds() > 0 {
            thread::sleep(Duration::from_secs(duration.num_seconds() as u64));
        }
        send_notification(message);
    }
}




fn make_schedule_expr(vec_to_parse: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut schedule_exprs = Vec::new();

    for (time_str, activity) in vec_to_parse {
        // Split the time string into hours and minutes
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            eprintln!(
                "Invalid time format for activity: {}, time: {}",
                activity, time_str
            );
            continue;
        }
        let hours: u32 = match parts[0].parse() {
            Ok(h) => h,
            Err(_) => {
                eprintln!(
                    "Invalid hours for activity: {}, time: {}",
                    activity, time_str
                );
                continue;
            }
        };
        let minutes: u32 = match parts[1].parse() {
            Ok(m) => m,
            Err(_) => {
                eprintln!(
                    "Invalid minutes for activity: {}, time: {}",
                    activity, time_str
                );
                continue;
            }
        };
// Ensure hours and minutes are within valid ranges
        if hours < 24 && minutes < 60 {
            let cron_expr = format!("0 {} {} * * *", minutes, hours);
            schedule_exprs.push((cron_expr, activity));
        } else {
            eprintln!(
                "Invalid time format for activity: {}, time: {}",
                activity, time_str
            );
        }
    }
    schedule_exprs
}
//         NOTE: END OF EXPRESSION MAKER FUNCTION






fn set_plan_notifs() {
    let mut txt_files: Vec<String> = fs::read_dir(".")
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.path().extension().unwrap_or_default() == "txt" {
                Some(entry.file_name().into_string().unwrap())
            } else {
                None
            }
        })
        .collect();
    txt_files.push("Go back".to_string());
    loop {
        let selection = dialoguer::Select::new()
            .with_prompt("Select a FILE to IMPORT schedule from")
            .items(&txt_files)
            .default(0)
            .interact()
            .unwrap();
        if txt_files[selection] == "Go back" {
            break;
        } else {
            println!("Type your TIMEZONE OFFSET from UTC:");
            let mut time_zone = String::new();
            stdin().read_line(&mut time_zone).unwrap();
            let time_change: i32 = match time_zone.trim().parse() {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("Invalid timezone input. Please enter a valid integer.");
                    continue;
                }
            };
            let time_change_minutes = time_change * 100;
            let filename: &str = txt_files[selection].as_str();
            let database: SimpleDB = SimpleDB::find_database(filename);
            let vec_to_disp = load_vec_from_db(database);
            let fixed_vec = fix_timezone(time_change_minutes, vec_to_disp);
            let parsed_vec = parse_time(fixed_vec);
            let expressions = make_schedule_expr(parsed_vec);


            let handles: Vec<_> = expressions
                .into_iter()
                .map(|(time, activity)| {
                    let schedule_expr = time.clone();
                    let activity_message = activity.clone();
                    thread::spawn(move || {
                        schedule_notifications(&schedule_expr, &activity_message);
                    })
                })
                .collect();
            // Wait for all threads to finish
            for handle in handles {
                let _ = handle.join();
            }
        }
    }
}
// NOTE: END OF NOTIFICATION ACTIVATOR FUNCTION


//--------------------------------------------------------------------------------






fn sort_vec(tuples_vec: &mut Vec<(u32, String)>) -> Vec<(u32, String)> {
    tuples_vec.sort_by_key(|tuple| tuple.1.clone());
    tuples_vec.to_vec()
}

fn save_vec_to_db(tuples_vec: &Vec<(u32, String)>) {
    println!("Type name of the schedule:");

    let mut timeblockname: String = String::new();
    stdin().read_line(&mut timeblockname).unwrap();
    let filename = format!("{}.txt", timeblockname.trim());

    let mut database = SimpleDB::find_database(&filename);
    for (time, activity) in tuples_vec {
        database.insert_into_db(time.to_string(), activity.clone().to_string());
    }
}

fn load_vec_from_db(database: SimpleDB) -> Vec<(u32, String)> {
    let data_vec: Vec<(String, String)> = database.data.into_iter().collect();
    let mut parsed_vec: Vec<(u32, String)> = Vec::new();
    for (time, activity) in data_vec {
        let time: u32 = time.parse().unwrap();
        let tuple = (time, activity);
        parsed_vec.push(tuple);
    }
    let sorted_vec = sort_vec(&mut parsed_vec);
    sorted_vec
}








fn run_menu() {
    let options = vec![
        "Create new schedule",
        "Display existing schedule",
        "Activate notification schedule",
        "Remove a schedule from list",
        "Quit application (Will disable notifications)",
    ];

    loop {
        let selection = Select::new()
            .with_prompt("Please select an option")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => {
                new_plan();
            }
            1 => {
                display_existing();
            }
            2 => {
                set_plan_notifs();
            }
            4 => {
                println!("Quitting application...");
                break;
            }
            3 => {
                remove_file();
            }
            _ => {
                println!("ERROR: invalid selection");
            }
        }
    }
}








fn remove_file() {
    let mut txt_files: Vec<String> = fs::read_dir(".")
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.path().extension().unwrap_or_default() == "txt" {
                Some(entry.file_name().into_string().unwrap())
            } else {
                None
            }
        })
        .collect();
    txt_files.push("Go back".to_string());
    loop {
        let selection = dialoguer::Select::new()
            .with_prompt("Select a file to DELETE")
            .items(&txt_files)
            .default(0)
            .interact()
            .unwrap();
        if txt_files[selection] == "Go back" {
            break;
        } else {
            let file_path: &str = txt_files[selection].as_str();

            // Attempt to delete the file
            match fs::remove_file(file_path) {
                Ok(_) => println!("File '{}' deleted successfully.", file_path),
                Err(e) => eprintln!("Failed to delete file '{}': {}", file_path, e),
            } // close match 
        } // close else
    } // close loop
} // close func










fn new_plan() {
    let mut act_to_time_vec: Vec<(u32, String)> = Vec::new();
    loop {
        let mut chosen_time = String::new() ;
        println!("Type time in format 'HH:MM' (we dont do PM); or type 'q' to save and go back; ");
        stdin().read_line(&mut chosen_time).unwrap();
        let selection = chosen_time.trim().to_string(); 

        if selection == "q".to_string() {
            save_vec_to_db(&act_to_time_vec);
            break;
        }

        let test_parts: Vec<&str> = selection.split(':').collect();
        if test_parts.len() != 2 || selection.len() != 5 {
            eprintln!("Invalid time format for activity");
        }

        let time_num: u32 = selection.replace(":", "").parse().unwrap();
        println!("Enter name of your activity: ");
        let mut activity: String = String::new();
        stdin().read_line(&mut activity).unwrap();

        if let Some((index, _)) = act_to_time_vec
            .iter()
            .enumerate()
            .find(|(_, tuple)| tuple.0 == time_num)
        {
            act_to_time_vec[index] = (time_num, activity.trim().to_string());
        } else {
            let tuple = (time_num, activity.trim().to_string());
            act_to_time_vec.push(tuple);
        }
    } // end of loop
}








fn display_existing() {
    let mut txt_files: Vec<String> = fs::read_dir(".")
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.path().extension().unwrap_or_default() == "txt" {
                Some(entry.file_name().into_string().unwrap())
            } else {
                None
            }
        })
        .collect();
    txt_files.push("Go back".to_string());
    loop {
        let selection = dialoguer::Select::new()
            .with_prompt("Select a file to import")
            .items(&txt_files)
            .default(0)
            .interact()
            .unwrap();
        if txt_files[selection] == "Go back" {
            break;
        } else {
            let filename: &str = txt_files[selection].as_str();
            let database: SimpleDB = SimpleDB::find_database(filename);
            let vec_to_disp = load_vec_from_db(database);
            let parsed_vec = parse_time(vec_to_disp);

            let max_len = parsed_vec
                .iter()
                .map(|(_, activity)| activity.len())
                .max()
                .unwrap_or(0);

            for (time, activity) in parsed_vec {
                let spaces = max_len - activity.len();
                println!("{}{} | {} ", activity, " ".repeat(spaces), time);
            }
        }
    }
}





fn parse_time(vec_to_parse: Vec<(u32, String)>) -> Vec<(String, String)> {
    let mut parsed_vec = Vec::new();
    for (time, item) in vec_to_parse {
        let time_string = time.to_string();
        let mut parsed_time = String::new();
        if time_string.len() == 1 {
            parsed_time = format!("0:0{}", time_string);
        } else if time_string.len() == 2 {
            parsed_time = format!("0:{}", time_string);
        } else if time_string.len() == 3 {
            parsed_time = format!("{}:{}", &time_string[0..1], &time_string[1..3]);
        } else if time_string.len() == 4 {
            parsed_time = format!("{}:{}", &time_string[0..2], &time_string[2..4]);
        }
        parsed_vec.push((parsed_time, item));
    }
    parsed_vec
}















