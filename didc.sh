#!/usr/bin/env bash

output_file=""
other_args=()

# Parse arguments
while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --output)
      output_file="$2"
      shift 2
      ;;
    *)
      # Collect other arguments
      other_args+=("$1")
      shift
      ;;
  esac
done

if [ -x ~/bin/didc ]
then
  ~/bin/didc ${other_args[*]} > ${output_file}
  exit $?
fi

check_didc() {
  command -v didc > /dev/null 2>&1
}

if check_didc
then
  didc "${@}"
  exit $?
fi
unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     machine=Linux;;
    Darwin*)    machine=Mac;;
    *)          machine="UNKNOWN:${unameOut}"
esac

release=$(curl --silent "https://api.github.com/repos/dfinity/candid/releases/latest" | grep -e '"tag_name"' | cut -c 16-25)

if [ ${machine} = "Mac" ]
then
  mkdir -p ~/bin
  curl -fsSL https://github.com/dfinity/candid/releases/download/${release}/didc-macos -o ~/bin/didc
  chmod +x ~/bin/didc
  ~/bin/didc ${other_args[*]} > ${output_file}
  exit $?
elif [ ${machine} = "Linux" ]
then
  mkdir -p ~/bin
  curl -fsSL https://github.com/dfinity/candid/releases/download/${release}/didc-linux64 -o ~/bin/didc
  chmod +x ~/bin/didc
  ~/bin/didc ${other_args[*]} > ${output_file}
  exit $?
else
  2> echo "Could not detect a supported operating system. Please note that didc is currently only supported for Mac and Linux"
fi