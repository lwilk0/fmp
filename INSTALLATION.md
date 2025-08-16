# Installation

## Dependencies

### Arch Linux
Install [fmp-git](https://aur.archlinux.org/packages/fmp-git)/[fmp-bin](https://aur.archlinux.org/packages/fmp-bin) from the AUR

OR

```
sudo pacman -S rustup libgpg-error gpgme 
rustup default stable
```

### Debian/Ubuntu
```
sudo apt install rustup libgpg-error-dev libgpgme-dev
rustup default stable
```

### Fedora/RHEL/CentOS
```
sudo dnf install rustup libgpg-error gpgme
rustup default stable
```

### Gentoo
```
emerge --ask dev-util/rustup dev-libs/libgpg-error app-crypt/gpgme
rustup default stable
```

### MacOS
```
brew install rustup libgpg-error gpgme
rustup default stable
```

### NixOS
FMP currently does not work as "libgpg-error" is never detected, even when installed

### openSUSE
```
sudo zypper install rustup libgpg-error gpgme
rustup default stable
```

### Windows
- Download and install rustup - [64bit](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) / [32bit](https://static.rust-lang.org/rustup/dist/i686-pc-windows-msvc/rustup-init.exe)
- Download and install [Gpg4win](https://gpg4win.org/thanks-for-download.html)
