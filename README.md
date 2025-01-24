# fmp

A command line password manager written in rust for Linux.

## Installation:
1. **Clone the repo:**
```bash
git clone https://github.com/TT1882/fmp.git
cd fmp
```
2. **Build and install fmp with cargo:**
```bash
cargo build --release
cargo install --path .
```

## Flags:
```flags
  -a, --add              Add an account to vault. used as: -a, --add
  -d, --delete           Delete account from vault. used as: -d, --delete
  -p, --change-password  Change password in account. used as: -p , --change-password
  -u, --change-username  Change username in account. used as: -u , --change-username
  -c, --create-vault     Create vault. used as -c --create-vault
  -e, --entropy          Calculate password entropy. used as -e --entropy
  -h, --help             Print help
```
###

## How fmp Works:
Fmp works by reading an encrypted folder (.fmpVault). Within this folder is a accounts file and multiple other folders, each containing a data.json file. Each file has the name of the account it represents (e.g. GitHub), the accounts file contains all of these account names, and each data.json contains the username and password of the corrosponding account. The folder is encrypted with Gnu Privacy Guard (GPG) after being compressed into a Tar/GZip archive.
### Read Vault:
The vault is read by first decrypting the .fmpVault.tar.gz.gpg file using the GPG command line utility, then the tarball is decompressed, leaving the plain .fmpVault folder behind. The accounts file is read and each data.json's password and username is read acording to the accounts list. All files par the encrypted tarball are removed and the program is exited.
### Add Account:
The vault is decrypted and the user is prompted to enter the account name, username and password. A new folder with the account name is created and a data.json file is made within. The username and password are added into the json file, and the account name is entered into the accounts file. The old encrypted vault is removed and the new one is encrypted in its place.
### Delete Account:
The vault is decrypted and the user is prompted to enter the name of the account they want to delete. The file with the corrosponding name is deleted from the vault and the entry with the the inputed account name is removed from the account file. The old encrypted vault is removed and the new one is encrypted in its place.
### Change Username:
The vault is decrypted and the user is propted to enter the name of the account and the password they want to change to. The data.json file is removed and replaced with data containing the new password is put in the same place. The old encrypted vault is removed and the new one is encrypted in its place.
### Change Password:
The vault is decrypted and the user is propted to enter the name of the account and the username they want to change to. The data.json file is removed and replaced with data containing the new username is put in the same place. The old encrypted vault is removed and the new one is encrypted in its place.
### Create Vault:
A .fmpVault folder is saved in the users home directory, along with an accounts file inside it. The vault is then compressed with Tar/GZip and encrypted.
### Calculate Entropy:
The user is prompted to enter the password they want to find the entropy of. The character pool size of the password is found and the entropy is calculated with the formula L * log2(C), where L is the length of the password and C is the character pool. The entropy is ranked and displayed.
