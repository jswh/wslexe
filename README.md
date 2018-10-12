## WSLEXE
The WSL system makes a great convenience of development on Windows.But the integretion to IDEs and code editors is not convenient.

The project [wslgit](https://github.com/andy-5/wslgit) makes a dummy exe that trying to receive the arguments and translating all paths from windows type to unix type, and reform these arguments to the real command in wsl .

This shows a way to use wsl application in windows enviroment for IDEs and code editors. But the project is made for git only. I made a bit change using the file stem name as the real command. And it works!

## Usage

1. download the [wslexe.exe](https://github.com/jswh/wslexe/releases)
2. rename it to the command you want to use, for example pyhon.exe
3. change your ide or editor config to point to the executable file
4. if you have a ".wslexerc" in the path where the executable file exists, it will be sourced before the real command

## Compatibility
* python.exe
  - [x] vscode
  - [x] powershell
* composer.exe
  - [x] phpstorm
* php.exe
  - [x] phpstorm
* git.exe
  - [x] vscode
  - [x] powershell

## Screen shot
#### python.exe for vscode
![show](https://user-images.githubusercontent.com/6405755/41839420-caa53562-7895-11e8-8ff8-576c56d9ba7c.gif)
