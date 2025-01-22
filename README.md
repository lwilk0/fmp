# fmp
A command line password manager written in rust for Linux, currently being re-written as code is unsafe and unreadable.
<video src="https://github.com/user-attachments/assets/760bba84-b978-4071-9252-167a1a09a883" controls="controls" style="max-width:150px;"></video>

###

## How fmp Works:
- [Decrypt and read](https://github.com/TT1882/fmp/blob/main/README.md#decrypt-and-read)
- [Add data to encrypted file](https://github.com/TT1882/fmp/blob/main/README.md#add-data-to-encrypted-file)
- [Remove data from encrypted file](https://github.com/TT1882/fmp/blob/main/README.md#remove-data-from-encrypted-file)
- [Backup all files and install backup](https://github.com/TT1882/fmp/blob/main/README.md#backup-all-files-and-install-backup)
- [Create a password](https://github.com/TT1882/fmp/blob/main/README.md#create-a-password)
- [Calculate password entropy](https://github.com/TT1882/fmp/blob/main/README.md#calculate-password-entropy)
- [Setup fmp](https://github.com/TT1882/fmp/blob/main/README.md#setup-fmp)

###

### Decrypt And Read: 
Fmp interacts with a json file containing account names and passwords, encrypted with the Advanced Encryption Standard with a 256-bit key in Cipher Block Chaining(aes-256-cbd). The file is also salted and uses Password-Based Key Derivation Function 2. When fmp is ran, it decrypts this file(secrets.json.enc) and the usernames are compared with a persistent file(accounts) which allows an output to be formated and displayed.

### Add Data To Encrypted File:
The secrets file is decrypted and the user is asked the username and password to the account they want to add. The "add" function is then called, which formats the inputs in json syntax and saves it to a variable. The "update_json" function is called next, appending the data to the full json data, then saves the data to the secrets file which is re-encrypted. The username is added to the accounts file.

### Remove Data From Encrypted File:
The secrets file is decrypted and the user is propted for the username and password to the account they want to remove. The inputs are compaired to the data within the secrets file through the "rem" function and if they match, the data corresponding to the inputs are removed and the file is re-encrypted. The accont name is removed from the accounts file.

### Backup All Files And Install Backup:
The user is promted on whether they want to backup files or install a backup. If a backup is selected all the files associated with fmp are coppied to a file in the same location with the prefix .bak e.g. account.bak. If an install is selected all the .bak files are coppied to there non .bak counterparts.

### Create A Password:
The user is first prompted to input the length of the password they want, then random characters from a password-permited charactes array are concatenated together to form a password of the length. The user is asked if they want to save it to an account. If so, an account name is inputed and the [add](https://github.com/TT1882/fmp/blob/main/README.md#add-data-to-encrypted-file) function is called.

### Calculate Password Entropy:
The user inputs the password to be rated. The presence of lowercase characters, uppercase characters, special characters and numbers and detected by the "char" function. If they are present, the number of the characters within the group are added to a variable(posSymbols). For example, if a lowercase character is present, 26 is added to the variable. The number of combinations avaliable for a password of its size and contents is calculated by puting the posSymbols variable to the power of the password length. The entropy is finaly calculated by finding the logarithm base 2 of the number of combinations.

### Setup Fmp:
Fmp is setup by simply asking where to save the accounts and secrets file, creating the directory if necessary, creating the directory pointer files at ~/.config/fmp and both the accounts and secrets file to there inputed location. Placeholder data is added to the secrets file to avoid errors before it is encrypted.

###

## Flags:
```flags
  -a, --add      Add an account to password manager. used as: -a, --add
  -b, --backup   Backup all fmp files or install backup. used as: -b, --backup
  -d, --delete   Delete account from password manager. used as: -d, --delete
  -s, --setup    Setup fmp. used as: -s, --setup
  -c, --create   Create password. used as: -c, --create
  -e, --entropy  Calculate password entropy. used as: -e, --entropy
  -h, --help     Print help
```
###

## TODO:
- [x] Add install function
- [x] u32 input error handling
- [x] Add error handling to directory input
- [x] Add backup function
- [x] Calculate password entropy
- [x] Ensure exit encryption
- [ ] Allow for entropy calculations larger than 19 characters
- [ ] Add Windows and Mac support
- [ ] Create TUI
- [ ] Add double encryption
- [ ] Create better decryption method
- [ ] Add support for preexisting files in install function
- [ ] Add username to account
