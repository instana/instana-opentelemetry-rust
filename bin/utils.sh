#!/usr/bin/env bash

#######################################
# IBM Confidential
# PID 5737-N85, 5900-AG5
# Copyright IBM Corp. 2025
#######################################

SUCCESS=0
ERROR=1

# It provides functions for pr management
force_success() {
  echo -n ""
}

# Echoes a sequence of commands and runs them if the state is not dryrun mode.
#
# Description:
# This function echoes the commands in the input first. If this script is not
# in dryrun mode, then the input is evaluated as a sequence of commands and
# those are executed. Pipes are allowed in the input.
# Enclose strings with escaped double quotes.
#
# Examples:
# run_cmd "find ./ -exec -H -n -i \"string\" {} \;| egrep \"string2\" "
# run_cmd "VARIABLE=\"$VARIABLE\""
#
# Returns:
# 0 if the script is in dryrun mode,
# otherwise the return code of the executed command sequence
#
run_cmd () {
  echo "$*";
  if [[ -n ${DRYRUN+x} && $DRYRUN -eq 1 ]]; then
    return 0;
  else
    eval "$@"; # substitute with "$@"
  fi
}

# Echoes the input if verbose mode is enabled.
#
# Description:
# This function echoes the whole input, only if the script is in verbose mode.
# Pipes are allowed in the input. Enclose strings with escaped double quotes.
#
# Examples:
# log "VARIABLE=\"$VARIABLE\""
# log "VARIABLE=\"$(echo $VARIABLE | sed \"s/string1/string2/g\")\""
#
log () {
  if [[ -n ${VERBOSE+x} && $VERBOSE -eq 1 ]]; then
    echo "$*";
  fi
}

# Setup the git configuration, enables commit and push
# Inputs: username, email, repo default branch
# It reads the global variable GH_ENTERPRISE_TOKEN
git_config_setup() {
  if [ $# -lt 3 ]; then
    echo "git_config_setup needs 3 input parameters: user, mail, default_branch"
    exit $ERROR;
  fi
  local username=$1
  local mail=$2
  local default_branch=$3

  # setting git config
  echo "setting up git config"
  git config user.name "$username"
  git config user.email "$mail"
  git config url."https://$GH_ENTERPRISE_TOKEN:x-oauth-basic@github.ibm.com/".insteadOf "https://github.ibm.com/"
  git config branch."$default_branch".remote origin
  git config branch."$default_branch".merge refs/heads/"$default_branch"
  git config remote.origin.fetch +refs/heads/*:refs/remotes/origin/*
}

