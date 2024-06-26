#!/usr/bin/zsh
echo "Script is: $0 running using $$ PID"
echo "Current shell used within the script is: `readlink /proc/$$/exe`"

script_shell="$(readlink /proc/$$/exe | sed "s/.*\///")"

echo -e "\nSHELL is = ${script_shell}\n" 

if [[ "${script_shell}" == "bash" ]]
then
    echo -e "\nI'm BASH\n"
fi