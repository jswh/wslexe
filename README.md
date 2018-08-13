## WSLEXE 
The WSL system make a convenience of development on Windows.But the integretion to IDEs and code editors is not convenient.

The project [wslgit](https://github.com/andy-5/wslgit) makes a dummy exe that trying to receive the arguments and translating all paths from windows type to unix type, and reform these arguments to the real command in wsl .

This shows a way to use wsl application in windows enviroment for ides or code editors. But the project is made for git only. I make a bit change using the file stem name as the real command. And it works!

## Usage

1. download the [wsl.exe](https://github.com/jswh/wslexe/releases/tag/v0.0.1)
2. rename it to the command you want to use, for example pyhon.exe
3. change your ide or editor config to point to the executable file

## Compatibility
* python.exe
  - [x] vscode
  - [x] powershell
* composer.exe
  - [x] phpstorm
* php.exe
  - [x] phpstorm

## Screen shot
#### python.exe for vscode
![show](https://user-images.githubusercontent.com/6405755/41839420-caa53562-7895-11e8-8ff8-576c56d9ba7c.gif)
