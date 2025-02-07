aaa# FMP (Forgot My Password)

A command line password manager written in rust for Linux.

## Installation:
1. **Prerequisites**

Before installing fmp, the following must be installed on your system:
- gpg
- cargo
- rust
- tar

2. **Clone the repo:**
```bash
git clone https://github.com/TT1882/Forgot-My-Password.git
cd Forgot-My-Password
```
3. **Build and install FMP with cargo:**
```bash
cargo build --release
cargo install --path .
```
#### Note:
Sometimes, this error will be thrown when running cargo install
```bash
warning: be sure to add '/home/dir/.cargo/bin' to your PATH to be able to run the installed binaries
```
To fix this, run:
```bash
export PATH=$PATH:~/.cargo/bin/
```

4. **Test for FMP:**

To test is FMP, run:
``` bash
fmp -c
```
This will prompt you to create a vault for FMP.

## Flags:
```flags
  -a, --add                    Add an account to vault. used as: -a, --add
  -b, --backup                 Backup vault or install backup user as -b, --backup
  -c, --create-vault           Create vault. used as -c --create-vault
  -C, --change-vault-password  Change vault password. used as -C --change-vault-password
  -d, --delete                 Delete account from vault. used as: -d, --delete
  -D, --delete-vault           Delete vault. used as: -D, --delete
  -e, --entropy                Calculate password entropy. used as -e --entropy
  -E, --encrypt                Encrypt vault. used as -E, --encrypt
  -g, --generate-password      Generate new password. used as -g --generate-password
  -p, --change-password        Change password for an account. used as: -p , --change-password
  -r, --rename-vault           Rename vault. used as: -r , --rename-vault
  -u, --change-username        Change username for an account. used as: -u , --change-username
  -h, --help                   Print help
```
###

## How FMP Works:
FMP works by reading an encrypted folder (.vault). Within this folder is a accounts file and multiple other folders, each containing a data.json file. Each folder has the name of the account it represents (e.g. GitHub), the accounts file contains all of these account names, and each data.json contains the username and password of the corresponding account. The folder is encrypted with Gnu Privacy Guard (GPG) after being compressed into a Tar/GZip archive.
### Read Vault:
The vault is read by first decrypting the .vault.tar.gz.gpg file using the GPG command line utility, then the tarball is decompressed, leaving the plain .vault folder behind. The accounts file is read and each data.json's password and username is read according to the accounts list. All files par the encrypted tarball are removed and the program is exited.
### Add Account:
The vault is decrypted and the user is prompted to enter the account name, username and password. A new folder with the account name is created and a data.json file is made within. The username and password are added into the json file, and the account name is entered into the accounts file. The old encrypted vault is removed and the new one is encrypted in its place.
### Delete Account:
The vault is decrypted and the user is prompted to enter the name of the account they want to delete. The file with the corresponding name is deleted from the vault and the entry with the the inputted account name is removed from the account file. The old encrypted vault is removed and the new one is encrypted in its place.
### Change Username:
The vault is decrypted and the user is propted to enter the name of the account and the password they want to change to. The data.json file is removed and replaced with data containing the new password is put in the same place. The old encrypted vault is removed and the new one is encrypted in its place.
### Change Password:
The vault is decrypted and the user is propted to enter the name of the account and the username they want to change to. The data.json file is removed and replaced with data containing the new username is put in the same place. The old encrypted vault is removed and the new one is encrypted in its place.
### Create Vault:
A .vault folder is saved in the users home directory, along with an accounts file inside it. The vault is then compressed with Tar/GZip and encrypted.
### Calculate Entropy:
The user is prompted to enter the password they want to find the entropy of. The character pool size of the password is found and the entropy is calculated with the formula L * log2(C), where L is the length of the password and C is the character pool. The entropy is ranked and displayed.
### Backup Vault:
The user is asked if they want to create or install a backup. If they choose to backup, then the vault.tar.gz.gpg is copied to a file called vault.tar.gz.gpg.bk. If a backup install is selected, the vault.tar.gz.gpg.bk is copied to the vault.tar.gz.gpg file.
### Create Vault:
The user is asked what they would like to call the vault. A folder with the inputted name appended with a dot is created in the users home directory and an accounts file is added to it. The folder is then encrypted.
### Delete Vault:
The user is asked what account they would like to remove, and the corrosponing vault is decrypted. All files relating to the vault are deleted.
### Generate Password:
The user is asked for the length of the password they want to generate. Random characters are generated and concatenated to form a password of the specified length. The user is asked if they would like to add it to an account
### Rename Vault:
Asks user what vault name to change and what they whant to change it to. Decrypts vault, changes its name then re-encrypts it. Old vault files are removed.
### Change Vault Password:
Simply decrypts user specified vault and encrypts with the password they want to change it to.
