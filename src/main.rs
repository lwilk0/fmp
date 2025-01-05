use clap::Parser;
use std::{path::Path, process::exit, fs, fs::File, io::Read, io};
use cmd_lib::run_cmd;
use rustc_serialize::json::Json;
use rand::Rng;
use simple_home_dir::*;

// Help table
#[derive(Debug, Parser)]
struct Options {
    /// required non-flag argument.

    /// Add an account to password manager.
    /// used as: -a, --add
    #[clap(short = 'a', long = "add")]
    flag_a: bool,

    /// Delete account from password manager.
    /// used as: -d, --delete
    #[clap(short = 'd', long = "delete")]
    flag_d: bool,

    /// Setup fmp.
    /// used as: -s, --setup
    #[clap(short = 's', long = "setup")]
    flag_s: bool,

    /// Create password.
    /// used as: -p, --create
    #[clap(short = 'p', long = "create")]
    flag_p: bool
}



fn main() {
    let pass_Chars = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','1','2','3','4','5','6','7','8','9','0','!','"','£','$','%','^','&','*','(',')','-','_','=','+','{','}','[',']','@',';',':','?','/','>','.','<',',','|'];
    let mut data = String::new();
    let mut input: String = String::new();
    let mut service: &str = "";
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
    let mut user: String = String::new();
    let mut acc: Vec<String> = vec![];
    let mut dir1: String = String::new();
    let mut dir2: String = String::new();
    let mut dir3: String = String::new();

    let opts = Options::parse();

    if opts.flag_s == true {
        println!("FMP SETUP");
        newline();
        println!("What user should fmp be installed for?");
        io::stdin()
            .read_line(&mut user);

        for i in 0..1 {
            user.pop();
        }
        user = format!("/home/{}", user);

        newline();

        println!("Three files will be stored in ~/.config/fmp, containing locations for other files");
        println!("Creating the directory...");
        run_cmd!(mkdir $user/.config/fmp);
        println!("Done!");

        newline();

        println!("DO NOT END DIRECTORIES WITH A / AND WRITE OUT THE FULL DIRECTORY (Dont use '~' or $HOME))");

        newline();

        println!("What directory should the encrypted password file (secrets.json.enc) go?");
        io::stdin()
            .read_line(&mut secretsLoc);

        newline();

        println!("What directory should the accounts file go?");
        io::stdin()
            .read_line(&mut accLoc);

        newline();

        for i in 0..1 {
            secretsLoc.pop();
            accLoc.pop();
        }

        println!("Creating the secrets.json directory...");
        run_cmd!(mkdir -p $secretsLoc);
        println!("Done");

        println!("Creating the accounts directory...");
        run_cmd!(mkdir -p $accLoc);
        println!("Done");

        newline();

        secretsEncLoc = format!("{}/secrets.json.enc", secretsLoc);
        secretsLoc = format!("{}/secrets.json", secretsLoc);
        accLoc = format!("{}/accounts", accLoc);

        dir1 = format!("{}/.config/fmp/secretsEncLoc", user);
        dir2 = format!("{}/.config/fmp/secretsLoc", user);
        dir3 = format!("{}/.config/fmp/accLoc", user);

        println!("Creating files in ~/config/fmp");
        fs::write(dir1, secretsEncLoc.clone()).expect("Could not save secrets.json.enc loaction file");
        fs::write(dir2, secretsLoc.clone()).expect("Could not save secrets.json location file");
        run_cmd!(touch $dir3);
        println!("Done");

        newline();

        println!("Creating secrets.json file");
        run_cmd!(touch $secretsLoc);
        println!("Done");
        println!("Creating accounts file");
        run_cmd!(touch $accLoc);
        println!("Done");

        newline();

        placeholder = "{\"placholderDoNotRemove\":\"placeholder\"}".to_string();
        fs::write(secretsLoc.clone(), placeholder).expect("Could not add placeholder to secrets.json");

        println!("Encrypting file");
        run_cmd!(openssl aes-256-cbc -a -salt -pbkdf2 -in $secretsLoc -out $secretsEncLoc).expect("Could not encrypt secrets.json");
        run_cmd!(rm $secretsLoc).expect("Could not remove secrets.json");
        println!("Done");
    }

    let mut home = home_dir().unwrap();
    home.to_str();

    let dir = format!("{}/.config/fmp/secretsLoc", home.display());
    let secrets_path: String = fs::read_to_string(dir).expect("Could not read file");

    let dir = format!("{}/.config/fmp/secretsEncLoc", home.display());
    let secrets_path_enc: String = fs::read_to_string("/home/wilko/.config/fmp/secretsEncLoc").expect("Could not read file");
    
    let dir = format!("{}/.config/fmp/accLoc", home.display());
    let acc_path: String = fs::read_to_string("/home/wilko/.config/fmp/accLoc").expect("Could not read file");


    //secrets.json path test
    let sd: bool = Path::new(&secrets_path_enc).exists();
    if sd == false {
        println!("secrets.json path does not exist, has it been created?");
        exit(0)
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

        // remove /n from account name and pass, dont know why parse wont work
        for i in 0..1 {
            addName.pop();
            addPassword.pop();
        }

        // adds data to json var and saves to secrets.json
        data = add(&addName, addPassword, json.to_string());
        update_json(secrets_path, data, secrets_path_enc);
        acc.push(addName);
        write_acc(&acc_path, &acc);

        //exit
        newline();
        println!("Account created successfully");
        exit(0);
    }


    // if flag -d or --delete is used
    if opts.flag_d == true {

        newline();
        println!("What account should be removed?");
        io::stdin()
            .read_line(&mut remName)
            .expect("Failed to read line");

        newline();

        println!("What is the password for the account?");
        io::stdin()
            .read_line(&mut remPassword)
            .expect("Failed to read line");

        // remove /n from account name and pass, dont know why parse wont work
        for i in 0..1 {
            remName.pop();
            remPassword.pop();
        }
        data = rem(&remName, &remPassword, json.to_string());
        update_json(secrets_path, data, secrets_path_enc);
        acc.retain(|acc| *acc != remName);
        write_acc(&acc_path, &acc);

        newline();
        exit(0);
    }
    

    // if flag -p or --create is used
    if opts.flag_p == true {

        // User input for password length
        println!("How many characters long should the password be?");
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Saves input string to length as unsigned integer
        let length: u8 = input.trim().parse().expect("Input not a (positive) integer");

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
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        newline();

        if input.to_lowercase().trim().to_string() == "y" {

            accountPassword = passwd;

            println!("What is the account name?");
            io::stdin()
                .read_line(&mut accountName)
                .expect("Failed to read line");
            
            // remove /n from account name, dont know why parse wont work
            for i in 0..1 {
                accountName.pop();
            }

            // adds data to json var and saves to secrets.json
            data = add(&accountName, accountPassword, json.to_string());
            update_json(secrets_path, data, secrets_path_enc);
            acc.push(accountName);
            write_acc(&acc_path, &acc);

            //exit
            newline();
            println!("Account created successfully");
            exit(0);
        } 
    }

    dele(secrets_path);
    table(service, &acc, json);

}

pub fn table<'a>(mut service: &'a str, acc: &'a Vec<String>, json: Json) {
    newline();
    for i in 0..acc.len() {
        service = acc[i].as_str();
        println!("{} password - {}", acc[i], rem_first_and_last(json[service].to_string()));
    }
}

pub fn decrypt(secrets_path: String, secrets_path_enc: String) {
    run_cmd!(openssl aes-256-cbc -d -a -pbkdf2 -in $secrets_path_enc -out $secrets_path).expect("Could not decrypt secrets.json");
}

pub fn encrypt(secrets_path: String, secrets_path_enc: String) {
    run_cmd!(rm $secrets_path_enc).expect("Could not remove secrets.json.enc");
    newline();
    println!("Enter password to re-encrypt file");
    newline();
    run_cmd!(openssl aes-256-cbc -a -salt -pbkdf2 -in $secrets_path -out $secrets_path_enc).expect("Could not encrypt secrets.json");
    dele(secrets_path);
}

// reads account file
pub fn read_acc(acc_path: String) -> Vec<String> {
    // Reads acc_path and saves as string to acc
    let accStr = fs::read_to_string(acc_path)
    .expect("Could not read accounts file");

    // Seperates each piece of data through the newline between and saves each word to vector acc
    let acc: Vec<String> = accStr.split('\n').map(|v| v.to_string()).collect();

    return acc;
}

pub fn write_acc(acc_path: &String, acc: &Vec<String>) {
    // Saves vector to accounts file, each piece of data seperated through newline
    fs::write(acc_path, acc.join("\n")).expect("Could not save accounts file");
}

// Json must be manipulated as a string, it will be converted to json later
pub fn add(accName: &String, accPassword: String, mut json: String) -> String {
    json.pop(); // remove last }
    // adds new object containing name and pass to json
    json = format!("{},\"{}\":\"{}\"}}", json, accName, accPassword);
    return json;
}

// Json must be manipulated as a string, it will be converted to json later
pub fn rem(accName: &String, accPassword: &String, mut json: String) -> String {
    // removes object from json
    let jsonDupeCheck: String = json.clone();
    let remString: String = format!(",\"{}\":\"{}\"", accName, accPassword);
    json = json.replace(remString.as_str(), "");
    if json == jsonDupeCheck {
        newline();
        println!("No object was removed, either username or password is incorrect");
    } else {
        println!("Account removed successfully")
    }
    return json;
}

pub fn update_json(secrets_path: String, json: String, secrets_path_enc: String) {
    fs::write(secrets_path.clone(), json);
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