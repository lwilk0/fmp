#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

use clap::Parser;
use std::{env, path::Path, process::exit, fs, fs::File, io::Read};
use cmd_lib::run_cmd;
use rustc_serialize::json::Json;
use rand::Rng;
use simple_home_dir::*;
use import_handle;
use ilog::IntLog;
use dir_input::{self, get_dir_input};

// Help table, displayed with flag -h or --help
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
    println!("DO NOT USE. Fmp is getting rewritten and this version is unsafe!!")
    // Init
    // Arrays for password creation and entropy calculation
    let pass_Chars = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','1','2','3','4','5','6','7','8','9','0','!','"','#','$','%','&','\'','(',')','*','+',',','-','.','/',':',';','<','=','>','?','@','[','\\',']','^','_','`','{','|','}','~'];
    let lower_Chars = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
    let upper_Chars = ['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z'];
    let number_chars = ['1','2','3','4','5','6','7','8','9','0'];
    let special_chars = ['!','"','#','$','%','&','\'','(',')','*','+',',','-','.','/',':',';','<','=','>','?','@','[','\\',']','^','_','`','{','|','}','~'];

    // Stores edited json file as string, first acts as a variable to save json file from computer
    let mut data: String = String::new();

    // Stores user input for questions that the answer does not need to be saved
    let mut input: String = String::new();
    
    // Stores generated password
    let mut generatedPassword: String = String::new();
    // Stores the user inputed length for password generation
    let mut length: u32 = 0;

    // Stores user inputed/generated name and password for account adding
    let mut accountName: String = String::new();
    let mut accountPassword: String = String::new();
    let mut addName: String = String::new();
    let mut addPassword: String = String::new();

    // Stores user inputed name and password for account removal
    let mut remName: String = String::new();
    let mut remPassword: String = String::new();

    // Stores file locations read from ~/.config/fmp
    let mut secretsEncLoc: String = String::new();
    let mut secretsLoc: String = String::new();
    let mut accLoc: String = String::new();

    // Stores placeholder for secrets.json
    let mut placeholder: String = String::new();

    // Stores all account names
    let mut acc: Vec<String> = vec![];
    // Stores account name currently being read
    let service: &str = "";

    // Stores location of each ~/.config/.fmp file to avoid hard coded directory
    let mut secretsEncLocDir: String = String::new();
    let mut secretsLocDir: String = String::new();
    let mut accLocDir: String = String::new();

    // Stores the user inputed password for entropy calculation
    let mut passwordRate: String = String::new();
    // Stores the character pool size for entropy calculation
    let mut charPoolSize: u128 = 0;
    // Stores the ammount of combinations possible for entropy password
    let mut posCombinations: u128 = 0;
    // Stores the entropy of the user inputed password
    let mut entropy: u128 = 0;
    // Stores password entropy ranking
    let mut rating: &str = "";
    
    // Stores flag user input bools
    let opts = Options::parse();

    // Stores home directroy
    let home = home_dir().unwrap();
    let homeDir = home.display();  

    //Stores location of fmp config folder
    let fmpDir = format!("{}/.config/fmp", homeDir);


    // Checks if current OS is linux, exits if not
    let os: &str =  env::consts::OS;  
    if os != "linux" {
        println!("fmp only runs on Linux, running on other OSes could be destructive");
        exit(1)
    }

    // If -s of --setup flag is used
    if opts.flag_s == true {
        
        input = import_handle::get_string_input("Continuing will remove any files located in ~/.config/fmp, do you want to continue? y n" );
        if input == "n" {
            exit(1)
        }
        
        // Remove all 3 pointer files if they exist
        if Path::new("{}/.config/fmp/accLoc").exists() {
            run_cmd!(rm $homeDir/.config/fmp/accLoc).expect("Failed to execute command")
        }
        if Path::new("{}/.config/fmp/secretsEncLoc").exists() {
            run_cmd!(rm $homeDir/.config/fmp/secretsEncLoc).expect("Failed to execute command")
        }
        if Path::new("{}/.config/fmp/secretsLoc").exists() {
            run_cmd!(rm $homeDir/.config/fmp/secretsLoc).expect("Failed to execute command")
        }

        println!("FMP SETUP");

        newline();

        println!("Three files will be stored in ~/.config/fmp, containing locations for other files");
        println!("Creating the directory...");

        // Check if fmp config folder exists, if it does not it is created
        let fmpDirCheck = Path::new(&fmpDir).exists();
        if fmpDirCheck == false {
            run_cmd!(mkdir $homeDir/.config/fmp).expect("Failed to execute command");
        }

        println!("Done!");

        newline();

        // Ask user where the secrets.json.enc file should go
        secretsLoc = get_dir_input("What directory should the encrypted password file (secrets.json.enc) go?");
    
        newline();

        // Ask user where the accounts file should go
        accLoc = get_dir_input("What directory should the accounts file go?");
 
        newline();

        // Format directory with file name for later use
        secretsEncLoc = format!("{}/secrets.json.enc", secretsLoc);
        secretsLoc = format!("{}/secrets.json", secretsLoc);
        accLoc = format!("{}/accounts", accLoc);

        // Format pointer file full directory
        secretsEncLocDir = format!("{}/.config/fmp/secretsEncLoc", homeDir);
        secretsLocDir = format!("{}/.config/fmp/secretsLoc", homeDir);
        accLocDir = format!("{}/.config/fmp/accLoc", homeDir);

        println!("Creating files in ~/config/fmp");

        // Creates ~/.config/fmp pointer files
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

        // Creates placeholder and addss it to secrets.json file
        placeholder = "{\"placholderDoNotRemove\":\"placeholder\"}".to_string();
        fs::write(secretsLoc.clone(), placeholder).expect("Could not add placeholder to secrets.json");

        println!("Encrypting file");
        encrypt(secretsLoc, secretsEncLoc);
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


    // Secrets.json path test, exits if it does not exist
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

    //If flag -a or --add is used
    if opts.flag_a == true {

        newline();

        // Ask user for account name
        addName = import_handle::get_string_input("What is the account name? ");

        newline();

        // ask user for password
        addPassword = import_handle::get_string_input("What is the account password? ");

        // adds data to json var and saves to secrets.json
        data = add(&addName, addPassword, json.to_string());
        update_json(secrets_path.clone(), data, secrets_path_enc);
        acc.push(addName);
        write_acc(&acc_path, &acc);

        //exit
        dele(&secrets_path);
        newline();
        println!("Account created successfully");
        exit(1);
    }

    // If flag -b or --backup is used
    if opts.flag_b == true {

        newline();

        // Input validation for backup or install
        while input != "b" && input != "i" {
            input = import_handle::get_string_input("Would you like to backup(b) or install backup(i)?");
            newline();
        }

        // If user wants to backup
        if input == "b" {

            // Backup location files by copying the files to one with the same name prefixed with .bak
            run_cmd!(cp $homeDir/.config/fmp/accLoc $homeDir/.config/fmp/accLoc.bak).expect("Failed to run command");
            run_cmd!(cp $homeDir/.config/fmp/secretsEncLoc $homeDir/.config/fmp/secretsEncLoc.bak).expect("Failed to run command");
            run_cmd!(cp $homeDir/.config/fmp/secretsLoc $homeDir/.config/fmp/secretsLoc.bak).expect("Failed to run command");
        
            // Remove location prefixes
            for i in 0..16{secrets_path_enc.pop();}
            for i in 0..8{acc_path.pop();}

            // Backup account and secrets file by copying the files to one with the same name prefixed with .bak
            run_cmd!(cp $secrets_path_enc/secrets.json.enc $secrets_path_enc/secrets.json.enc.bak).expect("Failed to run command");
            run_cmd!(cp $acc_path/accounts $acc_path/accounts.bak).expect("Failed to run command");
            println!("Done!");

            // Exit
            dele(&secrets_path);
            exit(1);
        }

        // If user wants to install backup
        else if input == "i" {

            // Install location files by copying .bak files and removing prefix
            run_cmd!(cp $homeDir/.config/fmp/accLoc.bak $homeDir/.config/fmp/accLoc).expect("Failed to run command");
            run_cmd!(cp $homeDir/.config/fmp/secretsEncLoc.bak $homeDir/.config/fmp/secretsEncLoc).expect("Failed to run command");
            run_cmd!(cp $homeDir/.config/fmp/secretsLoc.bak $homeDir/.config/fmp/secretsLoc).expect("Failed to run command");
        
            // Remove location prefixes
            for i in 0..16{secrets_path_enc.pop();}
            for i in 0..8{acc_path.pop();}

            // Install account and secrets file by copying .bak files and removing prefix
            run_cmd!(cp $secrets_path_enc/secrets.json.enc.bak $secrets_path_enc/secrets.json.enc).expect("Failed to run command");
            run_cmd!(cp $acc_path/accounts.bak $acc_path/accounts).expect("Failed to run command");
            println!("Done!");

            dele(&secrets_path);
            exit(1);
        }
    }

    // If flag -d or --delete is used
    if opts.flag_d == true {

        newline();

        // Ask user for account to be removed
        remName = import_handle::get_string_input("What account should be removed?");

        newline();

        // Ask user for password to account
        remPassword = import_handle::get_string_input("What is the password for the account?");

        // Remove account from json and accounts
        data = rem(&remName, &remPassword, json.to_string());
        update_json(secrets_path.clone(), data, secrets_path_enc);
        acc.retain(|acc| *acc != remName);
        write_acc(&acc_path, &acc);

        // Exit
        dele(&secrets_path);
        exit(1);
    }
    

    // if flag -c or --create is used
    if opts.flag_c == true {

        newline();

        // User input for password length
        length = import_handle::get_u32_input("How long should the password be? ");


        newline();

        // Generates password using by retriving random character from character array
        for i in 0..length {
            let randint: usize = rand::thread_rng().gen_range(1..=pass_Chars.len()-1);
            let randchar: char = pass_Chars[randint];
            generatedPassword = format!("{}{}", generatedPassword, randchar);
        }

        // Prints generated password
        println!("{}", generatedPassword);

        newline();

        // Resets input var
        let mut input: String = String::new();

        input = import_handle::get_string_input("Would you like to link this password to an account? y n");
        
        newline();

        // If user wants to link password to account
        if input.to_lowercase().trim().to_string() == "y" {

            accountPassword = generatedPassword;

            // Ask user for account name
            accountName = import_handle::get_string_input("What is the account name?");

            // adds data to json var and saves to secrets.json
            data = add(&accountName, accountPassword, json.to_string());
            update_json(secrets_path.clone(), data, secrets_path_enc);
            acc.push(accountName);
            write_acc(&acc_path, &acc);

            //exit
            dele(&secrets_path);
            newline();
            println!("Account created successfully");
            exit(1);
        } 
    }

    // If flag -e or --entropy used
    if opts.flag_e == true {

        newline();

        // Ask user for password to rate
        passwordRate = import_handle::get_string_input("Enter the password: ");
        
        // Handle passwords longer than 19 characters to avoid panic, this happens as the 128bit limit is exceeded when generating combination ammount
        while passwordRate.len() > 19 {
            println!("Integer must be less than 20 due to limitations with rust integer sizes");
            passwordRate = import_handle::get_string_input("Enter the password: ");
        }

        // Totals up character pool size
        charPoolSize += char(lower_Chars, &passwordRate);
        charPoolSize += char(upper_Chars, &passwordRate);
        charPoolSize += char(number_chars, &passwordRate);
        charPoolSize += char(special_chars, &passwordRate);
       
        // Calculate possible combinations
        posCombinations = charPoolSize.pow(passwordRate.len() as u32);
    
        // Calculates password entropy
        entropy = u128::log2(posCombinations as u128) as u128;

        // Gets password rating
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

        // Output
        println!("\nThere are {} combinations posible", posCombinations);
        println!("The password has {} bit entropy", entropy);
        println!("Password rating: {}", rating);

        // Exit
        encrypt(secrets_path, secrets_path_enc);
        exit(1)
    }

    // Removes unencrypted secrets.json
    dele(&secrets_path);

    // If acc contains any passwords
    if acc.len() != 0 {
        // Print password table
        table(service, &acc, json);
    }
    else {
        newline();
        println!("No accounts to display")
    }

}

// Prints password table
pub fn table<'a>(mut service: &'a str, acc: &'a Vec<String>, json: Json) {
   
    newline();
  
    // Prints line for each account and password 
    for i in 0..acc.len() {
        service = acc[i].as_str();
        println!("{} password - {}", acc[i], rem_first_and_last(json[service].to_string()));
    }
}

// Decrypts secrets.json.enc file
pub fn decrypt(secrets_path: String, secrets_path_enc: String) {

    // Decrypts file
    match run_cmd!(openssl aes-256-cbc -d -a -pbkdf2 -in $secrets_path_enc -out $secrets_path) {
        Ok(res) => return,
        // Handles bad decrypt to avoid panic
        Err(e) => decrypt(secrets_path, secrets_path_enc),
    }
}

// Encrypt secrets.json file
pub fn encrypt(secrets_path: String, secrets_path_enc: String) {
    
    // Removes secrets.json file
    dele(&secrets_path);
    
    newline();
   
    println!("Enter password to re-encrypt file");
    
    newline();
   
    // Encrypts file
    match run_cmd!(openssl aes-256-cbc -a -salt -pbkdf2 -in $secrets_path -out $secrets_path_enc) {
        Ok(res) => return,
        // Handles when user enters passwords that do not match, everything will break without this
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

// Writes acc vector to account file
pub fn write_acc(acc_path: &String, acc: &Vec<String>) {
    // Saves vector to accounts file, each piece of data seperated through newline
    fs::write(acc_path, acc.join("\n")).expect("Could not save accounts file");
}

// Json must be manipulated as a string
pub fn add(accName: &String, accPassword: String, mut json: String) -> String {
    // remove last }
    json.pop(); 

    // adds new object containing name and pass to json
    json = format!("{},\"{}\":\"{}\"}}", json, accName, accPassword);
    return json;
}

// Json must be manipulated as a string
pub fn rem(accName: &String, accPassword: &String, mut json: String) -> String {
    
    // Creates duplicate of json for later use
    let jsonDupeCheck: String = json.clone();

    // Formats json that must be removed
    let remString: String = format!(",\"{}\":\"{}\"", accName, accPassword);

    // Removes remString from json 
    json = json.replace(remString.as_str(), "");

    newline();

    // If nothing has changed in json
    if json == jsonDupeCheck {

        // Try removing from start
        let remString: String = format!("\"{}\":\"{}\",", accName, accPassword);
        json = json.replace(remString.as_str(), "");

        // If nothing has changed in json
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

// Writes edited json to secrets.json file
pub fn update_json(secrets_path: String, json: String, secrets_path_enc: String) {

    // Writes to secrets.json
    fs::write(secrets_path.clone(), json).expect("Could not write");

    // Encrypts secrets.json
    encrypt(secrets_path, secrets_path_enc);   
}

// Removes secrets.json
pub fn dele(secrets_path: &String) {
    run_cmd!(rm $secrets_path).expect("Could not remove secrets.json");
}

// Removes first and last character of a string
pub fn rem_first_and_last(mut stringR: String) -> String {

    // remove last character
    stringR.pop();      

    // If string still contains something
    if stringR.len() > 0 {
        // remove first character
        stringR.remove(0);  
    }

    return stringR;
}

pub fn newline() {
    println!("");
}

// Adds to character pool size
pub fn char<const N: usize>(chars: [char; N], passwordRate: &String) -> u128{
    // char test
    let mut i = 0;
    let mut charPoolSize: u128 = 0;

    // loop once for each character of the character set, unless a match is found
    while i < chars.len() {
        // Traverses through array until a match is met
        if passwordRate.contains(chars[i]) {
            // Ends while
            i = chars.len();
            
            // Adds value to pool size
            charPoolSize += chars.len() as u128;
        }
        i += 1;
    }
    return charPoolSize;
}
