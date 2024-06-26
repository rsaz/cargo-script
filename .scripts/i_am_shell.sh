#!/bin/bash

echo "Script is: $0 running using $$ PID"
echo "Current shell used within the script is: $(readlink /proc/$$/exe)"

script_shell="$(readlink /proc/$$/exe | sed 's/.*\///')"

echo -e "\nSHELL is = ${script_shell}\n"

if [[ "${script_shell}" == "bash" ]]; then
    echo -e "
    ____             _     
   | __ )  __ _ _ __| | __ 
   |  _ \ / _\` | '__| |/ / 
   | |_) | (_| | |  |   <  
   |____/ \__,_|_|  |_|\_\\ 
    I'm BASH
    "
elif [[ "${script_shell}" == "zsh" ]]; then
    echo -e "
    _____        _      
   |__  /___  __| | ___ 
     / // _ \/ _\` |/ _ \\
    / /|  __/ (_| |  __/
   /____\___|\__,_|\___|
    I'm ZSH
    "
elif [[ "${script_shell}" == "sh" ]]; then
    echo -e "
     ____  _     
    / ___|| |__  
    \___ \| '_ \ 
     ___) | | | |
    |____/|_| |_|
    I'm SH
    "
else
    echo -e "
    ___________           .___ 
    \_   _____/ ____    __| _/ 
     |    __)_ /    \  / __ |  
     |        \   |  \/ /_/ |  
    /_______  /___|  /\____ |  
            \/     \/      \/  
    Unknown shell: ${script_shell}
    "
fi
