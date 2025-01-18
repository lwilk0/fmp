# fmp
A command line password manager written in rust for Linux

## How fmp Works:
### Decrypt and read 
Fmp interacts with a json file containing account names and passwords, encrypted with the Advanced Encryption Standard with a 256-bit key in Cipher Block Chaining(aes-256-cbd). The file is also salted and uses Password-Based Key Derivation Function 2. When fmp is ran, it decrypts this file(secrets.json.enc) and the usernames are compared with a persistent file(accounts) which allows an output to be formated and displayed.

### Add data to Encrypted file
The secrets file is decrypted and the user is asked the username and password to the account they want to add. The "add" function is then called, which formats the inputs in json syntax and saves it to a variable. The "update_json" function is called next, appending the data to the full json data, then saves the data to the secrets file which is re-encrypted. The username is added to the accounts file.

### Remove data from encrypted file
The secrets file is decrypted and the user is propted for the username and password to the account they want to remove. The inputs are compaired to the data within the secrets file through the "rem" function and if they match, the data corresponding to the inputs are removed and the file is re-encrypted. The accont name is removed from the accounts file.

### Backup all files and install backup
The user is promted on whether they want to backup files or install a backup. If a backup is selected all the files associated with fmp are coppied to a file in the same location with the prefix .bak e.g. account.bak. If an install is selected all the .bak files are coppied to there non .bak counterparts.

### Creating a password
The user is first prompted to input the length of the password they want, then random characters from a password-permited charactes array are concatenated together to form a password of the length. The user is asked if they want to save it to an account. If so, an account name is inputed and the add function is called.

### Calculating password entropy
The user inputs the password to be rated. The presence of lowercase characters, uppercase characters, special characters and numbers and detected by the "char" function. If they are present, the number of the characters within the group are added to a variable(posSymbols). For example, if a lowercase character is present, 26 is added to the variable. The number of combinations avaliable for a password of its size and contents is calculated by puting the posSymbols variable to the power of the password length. The entropy is finaly calculated by finding the logarithm base 2 of the number of combinations.

#### TODO:
- [x] Add install function
- [x] u32 input error handling
- [x] Add error handling to directory input
- [x] Add backup function
- [x] Calculate password entropy
- [ ] Allow for entropy calculations larger than 19 characters
- [ ] Add Windows and Mac support
- [ ] Create TUI
- [ ] Add double encryption
- [ ] Create better decryption method
- [ ] Add support for preexisting files in install function
- [ ] Add username to account
