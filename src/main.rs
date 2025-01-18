#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

use clap::Parser;
use std::{env, path::Path, process::exit, fs, fs::File, io::Read, io};
use cmd_lib::run_cmd;
use rustc_serialize::json::Json;
use rand::Rng;
use simple_home_dir::*;
use import_handle::{self, get_string_input};
use ilog::IntLog;

// Help table
#[derive(Debug, Parser)]
struct Options {

    /// Add an account to password manager.
    /// used as: -a, --add
    #[clap(short = 'a', long = "add")]
    flag_a: bool,

    /// Backup all fmp files or install backup.
    /// used as: -b, --backup
    #[clap(short = 'b', long = "backup")]
    flag_b: bool,

    /// Delete account from password manager.
    /// used as: -d, --delete
    #[clap(short = 'd', long = "delete")]
    flag_d: bool,

    /// Setup fmp.
    /// used as: -s, --setup
    #[clap(short = 's', long = "setup")]
    flag_s: bool,

    /// Create password.
    /// used as: -c, --create
    #[clap(short = 'c', long = "create")]
    flag_c: bool,

    /// Calculate password entropy.
    /// used as: -e, --entropy
    #[clap(short = 'e', long = "entropy")]
    flag_e: bool
}


fn main() {
    // Init
    let pass_Chars = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','1','2','3','4','5','6','7','8','9','0','!','"','#','$','%','&','\'','(',')','*','+',',','-','.','/',':',';','<','=','>','?','@','[','\\',']','^','_','`','{','|','}','~'];
    let lower_Chars = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
    let upper_Chars = ['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z'];
    let number_chars = ['1','2','3','4','5','6','7','8','9','0'];
    let special_chars = ['!','"','#','$','%','&','\'','(',')','*','+',',','-','.','/',':',';','<','=','>','?','@','[','\\',']','^','_','`','{','|','}','~'];
    
    let mut data = String::new();
    let mut input: String = String::new();
    let service: &str = "";
    let mut passwd: String = String::new();
    let mut accountName: String = String::new();
    let mut accountPassword: String = String::new();
    let mut remName: String = String::new();
    let mut remPassword: String = String::new();
    let mut addName: String = String::new();
    let mut addPassword: String = String::new();
    let mut secretsEncLoc: String = String::new();
    let mut secretsLoc: String = String::new();
    let mut accLoc: String = String::new();
    let mut placeholder: String = String::new();
    let mut acc: Vec<String> = vec![];
    let mut secretsEncLocDir: String = String::new();
    let mut secretsLocDir: String = String::new();
    let mut accLocDir: String = String::new();
    let mut userInput: String = String::new();
    let mut passwordRate: String = String::new();
    let mut posSymbols: u128 = 0;
    let mut posCombinations: u128;
    let mut entropy: u128;
    let mut rating: &str;
    let opts = Options::parse();

    let home = home_dir().unwrap();
    let homeDir = home.display();  
    let fmpDir = format!("{}/.config/fmp", homeDir);
    let fmpDirCheck = Path::new(&fmpDir).exists();

    //os check
    let os: &str =  env::consts::OS;  

    if os != "linux" {
        println!("fmp only runs on Linux, running on other OSes could be destructive");
        exit(1)
    }

    if opts.flag_s == true {
        
        println!("FMP SETUP");
        newline();
        println!("Three files will be stored in ~/.config/fmp, containing locations for other files");
        println!("Creating the directory...");
        if fmpDirCheck == false {
            run_cmd!(mkdir $homeDir/.config/fmp).expect("Failed to execute command");
        }
        println!("Done!");

        newline();

        secretsLoc = get_dir_input("What directory should the encrypted password file (secrets.json.enc) go?");
    
        newline();

        accLoc = get_dir_input("What directory should the accounts file go?");
        
        println!("Creating the secrets.json directory...");
        run_cmd!(mkdir -p $secretsLoc).expect("Failed to execute command");
        println!("Done");

        println!("Creating the accounts directory...");
        run_cmd!(mkdir -p $accLoc).expect("Failed to execute command");
        println!("Done");

        newline();

        secretsEncLoc = format!("{}/secrets.json.enc", secretsLoc);
        secretsLoc = format!("{}/secrets.json", secretsLoc);
        accLoc = format!("{}/accounts", accLoc);

        secretsEncLocDir = format!("{}/.config/fmp/secretsEncLoc", homeDir);
        secretsLocDir = format!("{}/.config/fmp/secretsLoc", homeDir);
        accLocDir = format!("{}/.config/fmp/accLoc", homeDir);

        println!("Creating files in ~/config/fmp");
        fs::write(secretsEncLocDir, secretsEncLoc.clone()).expect("Could not save secrets.json.enc loaction file");
        fs::write(secretsLocDir, secretsLoc.clone()).expect("Could not save secrets.json location file");
        fs::write(accLocDir, accLoc.clone()).expect("Failed to execute command");
        println!("Done");

        newline();

        println!("Creating secrets.json file");
        run_cmd!(touch $secretsLoc).expect("Failed to execute command");
        println!("Done");
        println!("Creating accounts file");
        run_cmd!(touch $accLoc).expect("Failed to execute command");
        println!("Done");

        newline();

        placeholder = "{\"placholderDoNotRemove\":\"placeholder\"}".to_string();
        fs::write(secretsLoc.clone(), placeholder).expect("Could not add placeholder to secrets.json");

        println!("Encrypting file");
        run_cmd!(openssl aes-256-cbc -a -salt -pbkdf2 -in $secretsLoc -out $secretsEncLoc).expect("Could not encrypt secrets.json");
        run_cmd!(rm $secretsLoc).expect("Could not remove secrets.json");
        println!("Done");

        exit(1)
    }


    // Read location files from ~/.config/fmp and saves to variable
    let dir = format!("{}/.config/fmp/secretsLoc", homeDir);
    let secrets_path: String = fs::read_to_string(dir).expect("Could not read file");

    let dir = format!("{}/.config/fmp/secretsEncLoc", homeDir);
    let mut secrets_path_enc: String = fs::read_to_string(dir).expect("Could not read file");
    
    let dir = format!("{}/.config/fmp/accLoc", homeDir);
    let mut acc_path: String = fs::read_to_string(dir).expect("Could not read file");


    //secrets.json path test
    let sd: bool = Path::new(&secrets_path_enc).exists();
    if sd == false {
        println!("secrets.json path does not exist, has it been created?");
        exit(1);
    }  

    // Runs read_acc and saves to acc var
    acc = read_acc(acc_path.clone());

    // Decrypt secrets.json file
    decrypt(secrets_path.clone(), secrets_path_enc.clone());

    // Read secrets.json file
    let mut file = File::open(secrets_path.clone()).unwrap();
    
    // Saves file to json var as Json data type
    file.read_to_string(&mut data).unwrap();
    let mut json = Json::from_str(&data).unwrap();

    //if flag -a or --add is used
    if opts.flag_a == true {

        newline();
        println!("What should the account be named?");
        io::stdin()
            .read_line(&mut addName)
            .expect("Failed to read line");

        newline();

        println!("What is the password for the account?");
        io::stdin()
            .read_line(&mut addPassword)
            .expect("Failed to read line");

        // remove /n from account name and pass
        addName = addName.trim().to_string();
        addPassword = addPassword.trim().to_string();

        // adds data to json var and saves to secrets.json
        data = add(&addName, addPassword, json.to_string());
        update_json(secrets_path, data, secrets_path_enc);
        acc.push(addName);
        write_acc(&acc_path, &acc);

        //exit
        newline();
        println!("Account created successfully");
        exit(1);
    }

    // if flag -b or --backup is used
    if opts.flag_b == true {
        newline();
        while userInput != "b" && userInput != "i" {
            println!("Would you like to backup(b) or install backup(i)?");
            io::stdin().read_line(&mut userInput).expect("Failed to read line");
            userInput = userInput.trim().to_string();
            newline();
        }

        if userInput == "b" {
            // Backup location files
            run_cmd!(cp $homeDir/.config/fmp/accLoc $homeDir/.config/fmp/accLoc.bak).expect("Failed to run command");
            run_cmd!(cp $homeDir/.config/fmp/secretsEncLoc $homeDir/.config/fmp/secretsEncLoc.bak).expect("Failed to run command");
            run_cmd!(cp $homeDir/.config/fmp/secretsLoc $homeDir/.config/fmp/secretsLoc.bak).expect("Failed to run command");
        
            // Remove location prefixes
            for i in 0..16{secrets_path_enc.pop();}
            for i in 0..8{acc_path.pop();}

            // Backup account and secrets file
            run_cmd!(cp $secrets_path_enc/secrets.json.enc $secrets_path_enc/secrets.json.enc.bak).expect("Failed to run command");
            run_cmd!(cp $acc_path/accounts $acc_path/accounts.bak).expect("Failed to run command");
            println!("Done!");

            dele(secrets_path);
            exit(1);
        }
        else if userInput == "i" {
            // Install location files
            run_cmd!(cp $homeDir/.config/fmp/accLoc.bak $homeDir/.config/fmp/accLoc).expect("Failed to run command");
            run_cmd!(cp $homeDir/.config/fmp/secretsEncLoc.bak $homeDir/.config/fmp/secretsEncLoc).expect("Failed to run command");
            run_cmd!(cp $homeDir/.config/fmp/secretsLoc.bak $homeDir/.config/fmp/secretsLoc).expect("Failed to run command");
        
            // Remove location prefixes
            for i in 0..16{secrets_path_enc.pop();}
            for i in 0..8{acc_path.pop();}

            // Install account and secrets file
            run_cmd!(cp $secrets_path_enc/secrets.json.enc.bak $secrets_path_enc/secrets.json.enc).expect("Failed to run command");
            run_cmd!(cp $acc_path/accounts.bak $acc_path/accounts).expect("Failed to run command");
            println!("Done!");

            dele(secrets_path);
            exit(1);
        }
    }

    // if flag -d or --delete is used
    if opts.flag_d == true {

        newline();

        println!("What account should be removed?");
        io::stdin().read_line(&mut remName).expect("Failed to read line");

        newline();

        println!("What is the password for the account?");
        io::stdin().read_line(&mut remPassword).expect("Failed to read line");

       
        remName = remName.trim().to_string();
        remPassword = remPassword.trim().to_string();
        
        data = rem(&remName, &remPassword, json.to_string());
        update_json(secrets_path, data, secrets_path_enc);
        acc.retain(|acc| *acc != remName);
        write_acc(&acc_path, &acc);

        newline();
        exit(1);
    }
    

    // if flag -c or --create is used
    if opts.flag_c == true {

        newline();
        // User input for password length
        let length: u32  = import_handle::get_u32_input("How long should the password be? ");


        newline();

        // Generates password
        for i in 0..length {
            let randint: usize = rand::thread_rng().gen_range(1..=pass_Chars.len()-1);
            let randchar: char = pass_Chars[randint];
            passwd = format!("{}{}", passwd, randchar);
        }

        println!("{}", passwd);

        newline();

        // Resets input var
        let mut input: String = String::new();

        println!("Would you like to link this password to an account? y n");
        io::stdin().read_line(&mut input).expect("Failed to read line");

        newline();

        if input.to_lowercase().trim().to_string() == "y" {

            accountPassword = passwd;

            println!("What is the account name?");
            io::stdin().read_line(&mut accountName).expect("Failed to read line");
            
            // remove /n from account name
            accountName = accountName.trim().to_string();
            

            // adds data to json var and saves to secrets.json
            data = add(&accountName, accountPassword, json.to_string());
            update_json(secrets_path, data, secrets_path_enc);
            acc.push(accountName);
            write_acc(&acc_path, &acc);

            //exit
            newline();
            println!("Account created successfully");
            exit(1);
        } 
    }

    // If flag -e or --entropy used
    if opts.flag_e == true {

        newline();

        passwordRate = get_string_input("Enter the password: ");
        while passwordRate.len() > 19 {
            println!("Integer must be less than 20 due to limitations with rust integer sizes");
            passwordRate = get_string_input("Enter the password: ");
        }

        posSymbols += char(lower_Chars, &passwordRate);
        posSymbols += char(upper_Chars, &passwordRate);
        posSymbols += char(number_chars, &passwordRate);
        posSymbols += char(special_chars, &passwordRate);
        posCombinations = posSymbols.pow(passwordRate.len() as u32);
        entropy = u128::log2(posCombinations as u128) as u128;

        if entropy <= 35 {
            rating = "Very Weak"
        }
        else if entropy <= 59{
            rating = "Weak"
        }
        else if entropy <= 119{
            rating = "Strong"
        }
        else {
            rating = "Very Strong"
        }

        println!("\nThere are {} combinations posible", posCombinations);
        println!("The password has {} bit entropy", entropy);
        println!("Password rating: {}", rating);

        encrypt(secrets_path, secrets_path_enc);
        exit(1)
    }

    dele(secrets_path);
    if acc.len() != 0 {
        table(service, &acc, json);
    }
    else {
        println!("No accounts to display")
    }

}

pub fn table<'a>(mut service: &'a str, acc: &'a Vec<String>, json: Json) {
    newline();
    for i in 0..acc.len() {
        service = acc[i].as_str();
        println!("{} password - {}", acc[i], rem_first_and_last(json[service].to_string()));
    }
}

pub fn decrypt(secrets_path: String, secrets_path_enc: String) {
     match run_cmd!(openssl aes-256-cbc -d -a -pbkdf2 -in $secrets_path_enc -out $secrets_path) {
        Ok(res) => return,
        Err(e) => decrypt(secrets_path, secrets_path_enc),
     }
}

pub fn encrypt(secrets_path: String, secrets_path_enc: String) {
    dele(secrets_path.clone());
    newline();
    println!("Enter password to re-encrypt file");
    newline();
    match run_cmd!(openssl aes-256-cbc -a -salt -pbkdf2 -in $secrets_path -out $secrets_path_enc) {
        Ok(res) => return,
        Err(e) => encrypt(secrets_path, secrets_path_enc),
    }
}

// reads account file
pub fn read_acc(acc_path: String) -> Vec<String> {
    // Reads acc_path and saves as string to acc
    let mut accStr = fs::read_to_string(acc_path)
    .expect("Could not read accounts file");
    
    // Seperates each piece of data through the newline between and saves each word to vector acc
    let mut acc: Vec<String> = accStr.split('\n').map(|v| v.to_string()).collect();
    // Removes blank "" from acc
    acc.retain(|x| x != "");
    
    return acc;
}

pub fn write_acc(acc_path: &String, acc: &Vec<String>) {
    // Saves vector to accounts file, each piece of data seperated through newline
    fs::write(acc_path, acc.join("\n")).expect("Could not save accounts file");
}

// Json must be manipulated as a string
pub fn add(accName: &String, accPassword: String, mut json: String) -> String {
    json.pop(); // remove last }
    // adds new object containing name and pass to json
    json = format!("{},\"{}\":\"{}\"}}", json, accName, accPassword);
    return json;
}

// Json must be manipulated as a string
pub fn rem(accName: &String, accPassword: &String, mut json: String) -> String {
    // removes object from json
    let jsonDupeCheck: String = json.clone();
    let remString: String = format!(",\"{}\":\"{}\"", accName, accPassword);

    json = json.replace(remString.as_str(), "");

    newline();

    if json == jsonDupeCheck {
        let remString: String = format!("\"{}\":\"{}\",", accName, accPassword);
        json = json.replace(remString.as_str(), "");
        if json == jsonDupeCheck {
            println!("No account removed, did not match any data");
        }
        else {
            println!("Account removed successfully")
        }
    } else {
        println!("Account removed successfully")
    }
    return json;
}

pub fn update_json(secrets_path: String, json: String, secrets_path_enc: String) {
    fs::write(secrets_path.clone(), json).expect("Could not write");
    encrypt(secrets_path, secrets_path_enc);   
}
pub fn dele(secrets_path: String) {
    run_cmd!(rm $secrets_path).expect("Could not remove secrets.json");
}

pub fn rem_first_and_last(mut value: String) -> String {
    value.pop();      // remove last character
    if value.len() > 0 {
        value.remove(0);  // remove first character
    }
    return value;
}

pub fn newline() {
    println!("");
}

pub fn get_dir_input(message: &str) -> String {
    println!("{}", message);

    let mut userInput = String::new();
    let mut userInput2 = String::new();
    let mut lastPos: usize;
    let mut dirTest: bool;
    let home = home_dir().unwrap();
    let homeDir = home.display();  

    loop {
        // Gets user input for dir
        io::stdin().read_line(&mut userInput).expect("Failed to read line");


        lastPos = userInput.len()-1;

        // Dir format error handling
        if userInput.contains("~") {
            // If first character of string is a tilda
            if userInput.chars().next().unwrap() == '~' {
                userInput.remove(0);
                userInput = format!("{}{}", homeDir, userInput)
            }
            else {
                println!("Invalid location of tilda! Must be at the start of the directory!\n");
                get_dir_input(message);
            }
        }
        else if userInput.contains("$") {
            println!("Please do not use a environmental variable in the directory");
            newline();
            get_dir_input(message);
        }
        else if &userInput[..lastPos] == "/" {
            userInput.pop();
        }
        
        // Check if dir exists
        dirTest = Path::new(&userInput.trim()).exists();

        // If directory does not exist
        while dirTest == false{
            println!("Directory does not exist, would you like to create it? yn exit");
            io::stdin().read_line(&mut userInput2).expect("Failed to read line");

            if userInput2.trim().to_lowercase().to_string() == "y" {
                // Creates directory
                userInput = userInput.trim().to_string();
                run_cmd!(mkdir -p $userInput).expect("Dir not valid or needs superuser privileges to access")
            }
            else if userInput2.trim().to_lowercase().to_string() == "exit" {
                exit(1)
            }
            else {
                get_dir_input(message);
            }
            dirTest = Path::new(&userInput.trim()).exists();
        } 
        return userInput.trim().to_string();
    }
} 

pub fn char<const N: usize>(chars: [char; N], passwordRate: &String) -> u128{
    // char test
    let mut i = 0;
    let mut posSymbols: u128 = 0;

    while i < chars.len() {
        if passwordRate.contains(chars[i]) {
            i = chars.len();
            posSymbols += chars.len() as u128;
        }
        i += 1;
    }
    return posSymbols;
}