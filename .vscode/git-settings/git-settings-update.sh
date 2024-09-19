#!/bin/bash
# --------------------------------------------------------------------------------------------------
# -- Generates the .gitattributes and .gitignore files
# --------------------------------------------------------------------------------------------------

#? LAST SYNC: 2024-03-18 from repo bigdata-src

#region Message Helpers ----------------------------------------------------------------------------

prt() { printf '%b\e[0m' "$*" > /dev/stderr || true; };

prn() { printf '%b\e[0m\n' "$*" > /dev/stderr || true; };

msg() { printf '\e[92mINFO\e[0m: %b\e[0m\n' "$*" > /dev/stderr || true; };

wrn() { printf '\e[38;5;3mWARNING\e[0m: \e[38;5;229m%b\e[0m\n' "$*" > /dev/stderr || true; };

errm() { local n=$?; [ ${n} -gt 0 ] && local c="\e[38;5;229m #$n"; printf '\e[38;5;1mERROR%b\e[0m: \e[38;5;210m%b\e[0m\n' "${c:-}" "$*" > /dev/stderr || true; };

fail() { local n=$?; errm "$@" ; if test ${n} -ne 0 ; then exit "$n"; else exit 255; fi; };

#endregion -----------------------------------------------------------------------------------------

#region Colors -------------------------------------------------------------------------

export bold="\e[1m";   export dimmed="\e[2m";  export italic="\e[3m";
export under="\e[4m";  export blink="\e[5m";   export inverse="\e[7m";
export hidden="\e[8m"; export strike="\e[5m";  export normal="\e[22m\e[23m\e[24m\e[25m\e[27m\e[28m\e[29m";

export c0="\e[0m";      export f0="\e[39m";       export b0="\e[49m";       export s0="${normal}";
export c1="\e[38;5;4m"; export c2="\e[38;5;002m"; export c3="\e[38;5;006m"; export c4="\e[38;5;005m";
export e1="\e[38;5;1m"; export e2="\e[38;5;210m"; export e3="\e[38;5;204m"; export e4="\e[38;5;197m";
export w1="\e[38;5;3m"; export w2="\e[38;5;229m"; export w3="\e[38;5;222m"; export w4="\e[38;5;220m";
export a1="\e[38;5;5m"; export a2="\e[38;5;085m"; export a3="\e[38;5;199m"; export a4="\e[38;5;161m";
export g1="\e[38;5;0m"; export g2="\e[38;5;007m"; export g3="\e[38;5;248m"; export g4="\e[38;5;8m";

export C0="\e[0m";      export F0="\e[39m";       export B0="\e[49m";       export S0="${normal}";
export C1="\e[48;5;4m"; export C2="\e[48;5;002m"; export C3="\e[48;5;006m"; export C4="\e[48;5;005m";
export E1="\e[48;5;1m"; export E2="\e[48;5;210m"; export E3="\e[48;5;204m"; export E4="\e[48;5;197m";
export W1="\e[48;5;3m"; export W2="\e[48;5;229m"; export W3="\e[48;5;222m"; export W4="\e[48;5;220m";
export A1="\e[48;5;5m"; export A2="\e[48;5;085m"; export A3="\e[48;5;199m"; export A4="\e[48;5;161m";
export G1="\e[48;5;0m"; export G2="\e[48;5;007m"; export G3="\e[48;5;248m"; export G4="\e[48;5;8m";

export   black="\e[38;5;0m";   export    grey="\e[38;5;8m";  export     purple="\e[38;5;98m";
export     red="\e[38;5;196m"; export  maroon="\e[38;5;9m";  export   navyblue="\e[38;5;17m";
export   green="\e[38;5;2m";   export    lime="\e[38;5;10m"; export   darkblue="\e[38;5;18m";
export  yellow="\e[38;5;3m";   export   olive="\e[38;5;11m"; export     orange="\e[38;5;214m";
export    blue="\e[38;5;4m";   export    navy="\e[38;5;12m"; export lightgreen="\e[38;5;120m";
export magenta="\e[38;5;200m"; export fuchsia="\e[38;5;13m"; export  darkgreen="\e[38;5;22m";
export    cyan="\e[38;5;6m";   export    aqua="\e[38;5;14m"; export      coral="\e[38;5;204m";
export   white="\e[38;5;7m";   export  silver="\e[38;5;15m"; export       pink="\e[38;5;206m";

#endregion -----------------------------------------------------------------------------

#region Download -----------------------------------------------------------------------------------

#@ @1 -> URL to download
#@ @2 -> Path to copy the downloaded file.
download_to() {
    local title url dest;
    if [ $# -lt 2 ] ; then
        printf '\e[91m%s\e[0m requires 2 arguments but got only %s: %s' "$0" $# "$*" 1>&2||true;
        return 1;
    fi
    url="${1}"; dest="${2}"; title="${DOWNLOAD_NAME:-$(basename "${url}")}";
    printf 'Downloading and installing \e[94m%s\e[0m. Wait some seconds...' "${title}" 1>&2||true;
    ( if command -v curl > /dev/null; then
        curl -s -L -o "${dest}" "${url}";
    elif command -v wget > /dev/null; then
        wget -q -o "${dest}" "${url}";
    else
        printf 'Missing \e[91mwget\e[0m or \e[91mcurl\e[0m to download \e[93m%s\e[0m from \e[94m%s' "${title}" "${url}" 1>&2||true;
        return 2
    fi ) || printf '\e[91mFailed to download \e[93m%s\e[0m from \e[94m%s' "${title}" "${url}" 1>&2||true;
}

#@ @1  -> URL to download
# returns -> Path to temp file downloaded
download_from() {
    local file_url tmppath wurlfile downfile;
    file_url="${1:-}";
    tmppath="$(mktemp --directory --suffix='_downloaded/')";
    wurlfile="$(basename "${file_url:-}")"; downfile="${tmppath}${wurlfile:-}";
    if ! download_to "${file_url}" "${downfile}" ; then return $?; fi
    if ! test -f "${downfile}" ; then
        printf '\e[91mdownload_from\e[0m: Missing file downloaded from \e[94m%s\e[0m\n' "${file_url:-}" 1>&2||true;
        return 2;
    fi
    echo "${downfile}";
};

#@ @1 -> GIT URL to clone into local directory
#@ @2 -> Directory path used as parent for cloning the repository
clone_to(){
  if test -z "${2:-}"; then
    printf '\e[91mUsage\e[0m:\n\e[94m clone_to URL FOLDER\e[0m' 1>&2||true;
    return 10;
  fi
  if ! command -v git > /dev/null ; then
    printf '\e[91mFailed\e[0m: missing command \e[94mgit\e[0m installed and acessible in the PATH.' 1>&2||true;
    return 11;
  fi
  local git_repo_url cloned_dir;
  git_repo_url="${1}";
  if [[ "${2}" == "/"* ]] ; then cloned_dir="${2}"; else cloned_dir="${HOME}/${2:-}"; fi
  if ! test -d "${cloned_dir}"; then
    printf '\e[92mSetup\e[0m: cloning the git repository into \e[94m%s\e[0m\n' "${cloned_dir}" 1>&2||true;
    printf '\e[95m       Please wait some seconds...\e[0m\n' 1>&2||true;
    if ! mkdir -p "${cloned_dir}"; then
      printf '\n\e[93mFailed\e[0m: to create folder for the git repository in \e[94m%s\e[0m\n' "${cloned_dir}" 1>&2||true;
      return 12;
    elif ! git clone --quiet "${git_repo_url}" "${cloned_dir}" > /dev/stderr; then
      printf '\n\e[93mFailed\e[0m: to clone the git repository from \e[94m%s\e[0m into \e[94m%s\e[0m\n' "${git_repo_url}" "${cloned_dir}" 1>&2||true;
      return 13;
    fi
  fi
};

#@ @1 -> GIT URL to clone into local directory
# returns -> Directory path where is the cloned the repository
clone_from() {
    local tmppath gitrepo;
    tmppath="$(mktemp --directory --dry-run --suffix='_cloned_git_repo')";
    if ! clone_to "${1:-}" "${tmppath}" ; then return $?; fi
    # shellcheck disable=SC2012
    gitrepo=$(ls -a -1 "${tmppath}" | wc -l);
    if test "${gitrepo}" -lt 1; then
        printf '\e[91mclone_from\e[0m: Missing dir \e[94m%s\e[0m for repository cloned from \e[94m%s\e[0m\n' "${tmppath:-}" "${1:-}" 1>&2||true;
        return 21;
    fi
    echo "${tmppath}";
};

#endregion -----------------------------------------------------------------------------------------

#region Other Helpers ------------------------------------------------------------------------------

package_list_cleanup(){
    local fn;
    for fn in "$@"; do
        if ! test -f "${fn}"; then
            errm "Package list file not found: ${w1}${fn}";
            return 201;
        fi
    done
    for fn in "$@"; do
        msg "Getting package list from ${c1}${fn}${f0}...";
        if ! grep -vE '^\s*#|^\s*$' "${fn}"; then
            wrn "No package found in ${c1}${fn}${f0}.";
        fi
    done
};

get_git_root_dir(){ git rev-parse --show-toplevel; };

get_git_path() { git ls-files --full-name --cached --other "$@"; }

strpad(){
  local i;
  for ((i=1; i <= ${1}; i++)); do
    printf '%s' "${2:- }";
  done;
};

get_script_dir(){ cd -- "$(dirname "$0")" >/dev/null 2>&1 || exit 1 ; pwd -P || exit 2; };

get_script_name(){ basename "${ZSH_SOURCE:-${BASH_SOURCE:-$0}}"; };

get_script_path(){ cd -- "$(dirname "$0")" >/dev/null 2>&1 || exit 1 ; printf '%s/%s' "$(pwd -P)" "$(basename "${ZSH_SOURCE:-${BASH_SOURCE:-$0}}")" || exit 2; };

#endregion -----------------------------------------------------------------------------------------

#region Helpers ------------------------------------------------------------------------------------

clone_and_gen_settings_for(){
  local gitdir tmpdir gitfileext;
  if ! tmpdir=$(clone_from "${1}"); then errm "Error cloning: ${w1}"; return 220; fi

  if ! gitdir="$(get_git_root_dir)"; then fail "git-settings-update.sh must reside on a Git repository"; fi

  gitfileext="${2}";
  destfile="${gitdir}/.${gitfileext}";
  declare -a folders=( "${PWD}" "${get_script_dir}" "${gitdir}/.vscode/git-settings" "${gitdir}/.vscode" )

  for folder in "${folders[@]}"; do
    sourcefile="${folder}/${gitfileext:-gitignore}${3:-.txt}";
    msg "Looking up for: ${c2}${sourcefile}";
    if test -f "${sourcefile}"; then
      if gen_git_settings_for "${tmpdir}" "${gitfileext}" "${sourcefile}" "${destfile}";
      then msg "Finished settings file generation\n"; return 0;
      else return 221; fi
    fi
  done
  wrn "Missing settings file for language ${w3}${lang:-}${w1} in:\n${w2}${folders// /\n}";
}

#@ @1 -> Directory path used as parent for cloning the repository
#@ @2 -> Path of the language list file
gen_git_settings_for(){
  local sourcefile destfile gitfileext customfile;
  gitfileext="${2}";
  sourcefile="${3}";
  destfile="${4}";

  declare -a items;
  # shellcheck disable=SC2207
  items=( $(package_list_cleanup "${sourcefile}") ) || return 212;

  msg "Generating ${c1}.${gitfileext}${f0} settings from ${c3}${#items}${f0} types:\n  ${c2}${items[*]}";
  if ! read_git_settings_from "${1}" "${gitfileext}" "${sourcefile}" "${items[@]}" > "${destfile}"; then
    errm "Error generating ${e2}${gitfileext} ${e1} settings for file: ${w1}${fn}";
    return 213;
  fi

  customfile="$(dirname "${sourcefile}")/git-custom.${gitfileext}";
  if test -f "${customfile}"; then
    msg "Appending .${gitfileext} settings from: ${c2}${customfile}";
    cat "${customfile}" >> "${destfile}";
  fi
};

#@ @1 -> Directory path used as parent for cloning the repository
#@ @2 -> Language list
read_git_settings_from(){
  local srcpath gitfileext lang gitattfile wid;

  lang=$(printf '=%.0s' {1..99} | tr '=' '-')
  printf '#%s\n## .%s Settings\n#%s\n\n' "${lang}" "${2}" "${lang}";
  wid=$(get_git_path "$(get_script_path)");
  printf '#? This file was generated by the script %s\n' "${wid}";
  printf '#? from %s\n\n' "$(get_git_path "${3}")";

  srcpath="${1}"; gitfileext="${2}";
  for lang in "${@:4}"; do
    gitattfile="${lang}.${gitfileext}";
    gitattpath="${srcpath}/${gitattfile}";
    if ! test -f "${gitattpath}"; then
      wrn "Missing file ${w2}${gitattfile:-}${w1} for language ${w3}${lang:-}";
      continue;
    fi
    wid=${#lang}; wid=$(( 100 - wid - 9 ));
    printf '#region %s %s\n\n' "${lang}" "$(strpad ${wid} '-')";
    # Git automatically converts carriage return (\x0d) to line feed messing status
    grep -v -P '\x0d' < "${gitattpath}"; ## Printing output except ^M carrige return
    printf '\n#endregion '; printf '=%.0s' {1..89} | tr '=' '-'; printf '\n\n';
  done
};

#endregion -----------------------------------------------------------------------------------------

#region Initialization -----------------------------------------------------------------------------

clone_and_gen_settings_for "https://github.com/gitattributes/gitattributes.git" 'gitattributes';

clone_and_gen_settings_for "https://github.com/github/gitignore.git" 'gitignore';

#endregion -----------------------------------------------------------------------------------------
