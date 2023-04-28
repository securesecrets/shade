#!/bin/bash

set -eu
set -o pipefail # If anything in a pipeline fails, the pipe's exit status is a failure
#set -x # Print all commands for debugging

declare -a KEY=(a b c d)

declare -A FROM=(
    [a]='-y --from a'
    [b]='-y --from b'
    [c]='-y --from c'
    [d]='-y --from d'
)

# This means we don't need to configure the cli since it uses the preconfigured cli in the docker.
# We define this as a function rather than as an alias because it has more flexible expansion behavior.
# In particular, it's not possible to dynamically expand aliases, but `tx_of` dynamically executes whatever
# we specify in its arguments.
function secretcli() {
    docker exec secretdev /usr/bin/secretd "$@"
}

# Just like `echo`, but prints to stderr
function log() {
    echo "$@" >&2
}

# suppress all output to stdout for the command described in the arguments
function quiet() {
    "$@" >/dev/null
}

# suppress all output to stdout and stderr for the command described in the arguments
function silent() {
    "$@" >/dev/null 2>&1
}

# Pad the string in the first argument to 256 bytes, using spaces
function pad_space() {
    printf '%-256s' "$1"
}

function assert_eq() {
    set -e
    local left="$1"
    local right="$2"
    local message

    if [[ "$(echo $left | xargs)" != "$(echo $right | xargs)" ]]; then
        if [ -z ${3+x} ]; then
            local lineno="${BASH_LINENO[0]}"
            message="assertion failed on line $lineno - both sides differ. left: ${left@Q}, right: ${right@Q}"
        else
            message="$3"
        fi
        log "$message"
        return 1
    fi

    return 0
}

function assert_ne() {
    set -e
    local left="$1"
    local right="$2"
    local message

    if [[ "$left" == "$right" ]]; then
        if [ -z ${3+x} ]; then
            local lineno="${BASH_LINENO[0]}"
            message="assertion failed on line $lineno - both sides are equal. left: ${left@Q}, right: ${right@Q}"
        else
            message="$3"
        fi

        log "$message"
        return 1
    fi

    return 0
}

# what the fuck is this?
function assert_contains() {
    set -e
    local str="$1"
    local substr="$2"
    local message

    if [[ "$str" == "$substr" ]]; then
        if [ -z ${3+x} ]; then
            local lineno="${BASH_LINENO[0]}"
            message="assertion failed on line $lineno - str doesn't contain substr. str: ${str@Q}, substr: ${substr@Q}"
        else
            message="$3"
        fi
        log "$message"
        return 1
    fi

    return 0
}

declare -A ADDRESS=(
    [a]="$(secretcli keys show --address a)"
    [b]="$(secretcli keys show --address b)"
    [c]="$(secretcli keys show --address c)"
    [d]="$(secretcli keys show --address d)"
)

declare -A VK=([a]='' [b]='' [c]='' [d]='')

# Generate a label for a contract with a given code id
# This just adds "contract_" before the code id.
function label_by_id() {
    local id="$1"
    echo "contract_$id"
}

# Keep polling the blockchain until the tx completes.
# The first argument is the tx hash.
# The second argument is a message that will be logged after every failed attempt.
# The tx information will be returned.
function wait_for_tx() {
    local tx_hash="$1"
    local message="$2"

    local result

    log "waiting on tx: $tx_hash"
    # secretcli will only print to stdout when it succeeds
    until result="$(secretcli query tx "$tx_hash" 2>/dev/null)"; do
        log "$message"
        sleep 1
    done

    # log out-of-gas events
    if quiet jq -e '.raw_log | startswith("execute contract failed: Out of gas: ") or startswith("out of gas:")' <<<"$result"; then
        log "$(jq -r '.raw_log' <<<"$result")"
    fi

    echo "$result"
}

# This is a wrapper around `wait_for_tx` that also decrypts the response,
# and returns a nonzero status code if the tx failed
function wait_for_compute_tx() {
    local tx_hash="$1"
    local message="$2"
    local return_value=0
    local result
    local decrypted

    result="$(wait_for_tx "$tx_hash" "$message")"
    # log "$result"
    if quiet jq -e '.logs == null' <<<"$result"; then
        return_value=1
    fi
    decrypted="$(secretcli query compute tx "$tx_hash")" || return
    log "$decrypted"
    echo "$decrypted"

    return "$return_value"
}

# If the tx failed, return a nonzero status code.
# The decrypted error or message will be echoed
function check_tx() {
    local tx_hash="$1"
    local result
    local return_value=0

    result="$(secretcli query tx "$tx_hash")"
    if quiet jq -e '.logs == null' <<<"$result"; then
        return_value=1
    fi
    decrypted="$(secretcli query compute tx "$tx_hash")" || return
    log "$decrypted"
    echo "$decrypted"

    return "$return_value"
}

# Extract the tx_hash from the output of the command
function tx_of() {
    "$@" | jq -r '.txhash'
}

# Extract the output_data_as_string from the output of the command
function data_of() {
    local result="$("$@" | jq -ec '.answers[0].output_data_as_string' | sed 's/\\//g' | sed 's/ //g' | sed 's/^"\(.*\)"$/\1/')"
    echo "$result"
}

function extract_exec_error() {
    set -e
    local search_pattern
    local error_msg

    error_msg="$(jq -r '.output_error' <<<"$1")"
    search_pattern=${error_msg#*"$2"}
    echo $search_pattern
}

# Send a compute transaction and return the tx hash.
# All arguments to this function are passed directly to `secretcli tx compute execute`.
function compute_execute() {
    tx_of secretcli tx compute execute "$@"
}

# Send a query to the contract.
# All arguments to this function are passed directly to `secretcli query compute query`.
function compute_query() {
    secretcli query compute query "$@"
}

function upload_code() {
    set -e
    local directory="$1"
    local tx_hash
    local code_id

    tx_hash="$(tx_of secretcli tx compute store "code/$directory/contract.wasm.gz" ${FROM[a]} --gas 10000000)"
    code_id="$(
        wait_for_tx "$tx_hash" 'waiting for contract upload' |
            jq -r '.logs[0].events[0].attributes[] | select(.key == "code_id") | .value'
    )"

    log "uploaded contract #$code_id"

    echo "$code_id"
}

function instantiate() {
    set -e
    local code_id="$1"
    local init_msg="$2"

    log 'sending init message:'
    log "${init_msg@Q}"

    local tx_hash
    tx_hash="$(tx_of secretcli tx compute instantiate "$code_id" "$init_msg" --label "$(label_by_id "$code_id")" ${FROM[a]} --gas 10000000)"
    wait_for_tx "$tx_hash" 'waiting for init to complete'
}

# This function uploads and instantiates a contract, and returns the new contract's address
function create_contract() {
    set -e
    local dir="$1"
    local init_msg="$2"

    local code_id
    code_id="$(upload_code "$dir")"

    local init_result
    init_result="$(instantiate "$code_id" "$init_msg")"

    if quiet jq -e '.logs == null' <<<"$init_result"; then
        log "$(secretcli query compute tx "$(jq -r '.txhash' <<<"$init_result")")"
        return 1
    fi

    jq -r '.logs[0].events[0].attributes[] | select(.key == "contract_address") | .value' <<<"$init_result"
}

function deposit() {
    set -e
    local contract_addr="$1"
    local key="$2"
    local amount="$3"

    local deposit_message='{"deposit":{"padding":":::::::::::::::::"}}'
    local tx_hash
    local deposit_response
    tx_hash="$(compute_execute "$contract_addr" "$deposit_message" --amount "${amount}uscrt" ${FROM[$key]} --gas 250000)"
    deposit_response="$(wait_for_compute_tx "$tx_hash" "waiting for deposit to \"$key\" to process" | jq -ec '.answers[0].output_data_as_string' | sed 's/\\//g' | sed 's/ //g' | sed 's/^"\(.*\)"$/\1/')"
    assert_eq "$deposit_response" "$(pad_space '{"deposit":{"status":"success"}}' | sed 's/ //g')"
    log "deposited ${amount}uscrt to \"$key\" successfully"
    echo "$tx_hash"
}

function mint() {
    set -e
    local contract_addr="$1"
    local key="$2"
    local recipient="$3"
    local amount="$4"

    local mint_message='{"mint":{"recipient":"'"$recipient"'","amount":"'"$amount"'","padding":":::::::::::::::::"}}'
    local tx_hash
    local deposit_response
    tx_hash="$(compute_execute "$contract_addr" "$mint_message" ${FROM[$key]} --gas 251000)"
    echo "$tx_hash"
    deposit_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for mint to \"$recipient\" to process")"
    assert_eq "$deposit_response" "$(pad_space '{"mint":{"status":"success"}}' | sed 's/ //g')"
    log "minted ${amount}uscrt for \"$recipient\" successfully"
}

function burn() {
    set -e
    local contract_addr="$1"
    local key="$2"
    local amount="$3"

    local burn_message='{"burn":{"amount":"'"$amount"'"}}'
    local tx_hash
    local burn_response
    tx_hash="$(compute_execute "$contract_addr" "$burn_message" ${FROM[$key]} --gas 250000)"
    echo "$tx_hash"
    burn_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for burn for \"$key\" to process")"
    log "$burn_response"
    assert_eq "$burn_response" "$(pad_space '{"burn":{"status":"success"}}' | sed 's/ //g')"
    log "burned ${amount}uscrt for \"$key\" successfully"
}

function get_balance() {
    set -e
    local contract_addr="$1"
    local key="$2"

    log "querying balance for \"$key\""
    local balance_query='{"balance":{"address":"'"${ADDRESS[$key]}"'","key":"'"${VK[$key]}"'"}}'
    local balance_response
    balance_response="$(compute_query "$contract_addr" "$balance_query")"
    log "balance response was: $balance_response"
    jq -r '.balance.amount' <<<"$balance_response"
}

# Redeem some SCRT from an account
# As you can see, verifying this is happening correctly requires a lot of code
# so I separated it to its own function, because it's used several times.
function redeem() {
    set -e
    local contract_addr="$1"
    local key="$2"
    local amount="$3"

    local redeem_message
    local tx_hash
    local redeem_tx
    local transfer_attributes
    local redeem_response

    log "redeeming \"$key\""
    redeem_message='{"redeem":{"amount":"'"$amount"'","denom":"uscrt"}}'
    tx_hash="$(compute_execute "$contract_addr" "$redeem_message" ${FROM[$key]} --gas 150000)"
    redeem_tx="$(wait_for_tx "$tx_hash" "waiting for redeem from \"$key\" to process")"
    transfer_attributes="$(jq -r '.logs[0].events[] | select(.type == "transfer") | .attributes' <<<"$redeem_tx")"
    assert_eq "$(jq -r '.[] | select(.key == "recipient") | .value' <<<"$transfer_attributes")" "${ADDRESS[$key]}"
    assert_eq "$(jq -r '.[] | select(.key == "amount") | .value' <<<"$transfer_attributes")" "${amount}uscrt"
    log "redeem response for \"$key\" returned ${amount}uscrt"

    redeem_response="$(data_of check_tx "$tx_hash")"
    assert_eq "$redeem_response" "$(pad_space '{"redeem":{"status":"success"}}' | sed 's/ //g')"
    log "redeemed ${amount} from \"$key\" successfully"
    echo "$tx_hash"
}

function get_token_info() {
    set -e
    local contract_addr="$1"

    local token_info_query='{"token_info":{}}'
    compute_query "$contract_addr" "$token_info_query"
}

function increase_allowance() {
    set -e
    local contract_addr="$1"
    local owner_key="$2"
    local spender_key="$3"
    local amount="$4"

    local owner_address="${ADDRESS[$owner_key]}"
    local spender_address="${ADDRESS[$spender_key]}"
    local allowance_message='{"increase_allowance":{"spender":"'"$spender_address"'","amount":"'"$amount"'"}}'
    local allowance_response

    tx_hash="$(compute_execute "$contract_addr" "$allowance_message" ${FROM[$owner_key]} --gas 150000)"
    allowance_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for the increase of \"$spender_key\"'s allowance to \"$owner_key\"'s funds to process")"
    assert_eq "$(jq -r '.increase_allowance.spender' <<<"$allowance_response")" "$spender_address"
    assert_eq "$(jq -r '.increase_allowance.owner' <<<"$allowance_response")" "$owner_address"
    jq -r '.increase_allowance.allowance' <<<"$allowance_response"
    log "Increased allowance given to \"$spender_key\" from \"$owner_key\" by ${amount}uscrt successfully"
}

function decrease_allowance() {
    set -e
    local contract_addr="$1"
    local owner_key="$2"
    local spender_key="$3"
    local amount="$4"

    local owner_address="${ADDRESS[$owner_key]}"
    local spender_address="${ADDRESS[$spender_key]}"
    local allowance_message='{"decrease_allowance":{"spender":"'"$spender_address"'","amount":"'"$amount"'"}}'
    local allowance_response

    tx_hash="$(compute_execute "$contract_addr" "$allowance_message" ${FROM[$owner_key]} --gas 150000)"
    allowance_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for the decrease of \"$spender_key\"'s allowance to \"$owner_key\"'s funds to process")"
    assert_eq "$(jq -r '.decrease_allowance.spender' <<<"$allowance_response")" "$spender_address"
    assert_eq "$(jq -r '.decrease_allowance.owner' <<<"$allowance_response")" "$owner_address"
    jq -r '.decrease_allowance.allowance' <<<"$allowance_response"
    log "Decreased allowance given to \"$spender_key\" from \"$owner_key\" by ${amount}uscrt successfully"
}

function get_allowance() {
    set -e
    local contract_addr="$1"
    local owner_key="$2"
    local spender_key="$3"

    log "querying allowance given to \"$spender_key\" by \"$owner_key\""
    local owner_address="${ADDRESS[$owner_key]}"
    local spender_address="${ADDRESS[$spender_key]}"
    local allowance_query='{"allowance":{"spender":"'"$spender_address"'","owner":"'"$owner_address"'","key":"'"${VK[$owner_key]}"'"}}'
    local allowance_response
    allowance_response="$(compute_query "$contract_addr" "$allowance_query")"
    log "allowance response was: $allowance_response"
    assert_eq "$(jq -r '.allowance.spender' <<<"$allowance_response")" "$spender_address"
    assert_eq "$(jq -r '.allowance.owner' <<<"$allowance_response")" "$owner_address"
    jq -r '.allowance.allowance' <<<"$allowance_response"
}

# This function is the same as above, but it also checks the expiration
function check_allowance() {
    set -e
    local contract_addr="$1"
    local owner_key="$2"
    local spender_key="$3"
    local expiration="$4"

    log "querying allowance given to \"$spender_key\" by \"$owner_key\""
    local owner_address="${ADDRESS[$owner_key]}"
    local spender_address="${ADDRESS[$spender_key]}"
    local allowance_query='{"allowance":{"spender":"'"$spender_address"'","owner":"'"$owner_address"'","key":"'"${VK[$owner_key]}"'"}}'
    local allowance_response
    allowance_response="$(compute_query "$contract_addr" "$allowance_query")"
    log "allowance response was: $allowance_response"
    assert_eq "$(jq -r '.allowance.spender' <<<"$allowance_response")" "$spender_address"
    assert_eq "$(jq -r '.allowance.owner' <<<"$allowance_response")" "$owner_address"
    assert_eq "$(jq -r '.allowance.expiration' <<<"$allowance_response")" "$expiration"
    jq -r '.allowance.allowance' <<<"$allowance_response"
}

function log_test_header() {
    log " ########### Starting ${FUNCNAME[1]} ###############################################################################################################################################"
}

function extract_viewing_key_from_result() {
    set -e
    local tx_hash="$1"
    local key="$2"
    local viewing_key

    viewing_key_response="$(wait_for_compute_tx "$tx_hash" "waiting for viewing key for \"$key\" to be created")"
    viewing_key="$(jq -ec '.answers[0].output_data_as_string' <<<"$viewing_key_response" | cut -d'\' -f 6 | cut -c2-)"

    log "viewing key for \"$key\" set to ${viewing_key}"
    echo "$viewing_key"
}

function test_viewing_key() {
    set -e
    local contract_addr="$1"

    log_test_header

    # common variables
    local result
    local tx_hash

    # query balance. Should fail.
    local wrong_key
    wrong_key="$(xxd -ps <<<'wrong-key')"
    local balance_query
    local expected_error='{"viewing_key_error":{"msg":"Wrong viewing key for this address or viewing key not set"}}'
    for key in "${KEY[@]}"; do
        log "querying balance for \"$key\" with wrong viewing key"
        balance_query='{"balance":{"address":"'"${ADDRESS[$key]}"'","key":"'"$wrong_key"'"}}'
        result="$(compute_query "$contract_addr" "$balance_query")"
        assert_eq "$result" "$expected_error"
    done

    # Create viewing keys
    local create_viewing_key_message='{"create_viewing_key":{"entropy":"MyPassword123"}}'
    local viewing_key_response
    for key in "${KEY[@]}"; do
        log "creating viewing key for \"$key\""
        tx_hash="$(compute_execute "$contract_addr" "$create_viewing_key_message" ${FROM[$key]} --gas 1400000)"
        VK[$key]="$(extract_viewing_key_from_result "$tx_hash" "$key")"


        if [[ "${VK[$key]}" =~ ^api_key_ ]]; then
            log "viewing key \"$key\" seems valid"
        else
            log 'viewing key is invalid'
            return 1
        fi
    done

    # Check that all viewing keys are different despite using the same entropy
    assert_ne "${VK[a]}" "${VK[b]}"
    assert_ne "${VK[b]}" "${VK[c]}"
    assert_ne "${VK[c]}" "${VK[d]}"

    # query balance. Should succeed.
    local balance_query
    for key in "${KEY[@]}"; do
        balance_query='{"balance":{"address":"'"${ADDRESS[$key]}"'","key":"'"${VK[$key]}"'"}}'
        log "querying balance for \"$key\" with correct viewing key"
        result="$(compute_query "$contract_addr" "$balance_query")"
        if ! silent jq -e '.balance.amount | tonumber' <<<"$result"; then
            log "Balance query returned unexpected response: ${result@Q}"
            return 1
        fi
    done

    # Change viewing keys
    local vk2_a

    log 'creating new viewing key for "a"'
    tx_hash="$(compute_execute "$contract_addr" "$create_viewing_key_message" ${FROM[a]} --gas 1400000)"
    vk2_a="$(extract_viewing_key_from_result "$tx_hash" "$key")"
    assert_ne "${VK[a]}" "$vk2_a"

    # query balance with old keys. Should fail.
    log 'querying balance for "a" with old viewing key'
    local balance_query_a='{"balance":{"address":"'"${ADDRESS[a]}"'","key":"'"${VK[a]}"'"}}'
    result="$(compute_query "$contract_addr" "$balance_query_a")"
    assert_eq "$result" "$expected_error"

    # query balance with new keys. Should succeed.
    log 'querying balance for "a" with new viewing key'
    balance_query_a='{"balance":{"address":"'"${ADDRESS[a]}"'","key":"'"$vk2_a"'"}}'
    result="$(compute_query "$contract_addr" "$balance_query_a")"
    if ! silent jq -e '.balance.amount | tonumber' <<<"$result"; then
        log "Balance query returned unexpected response: ${result@Q}"
        return 1
    fi

    # Set the vk for "a" to the original vk
    log 'setting the viewing key for "a" back to the first one'
    local set_viewing_key_message='{"set_viewing_key":{"key":"'"${VK[a]}"'"}}'
    tx_hash="$(compute_execute "$contract_addr" "$set_viewing_key_message" ${FROM[a]} --gas 1400000)"
    viewing_key_response="$(wait_for_compute_tx "$tx_hash" "waiting for viewing key for "a" to be set")"
    viewing_key_response="$(jq -ec '.answers[0].output_data_as_string' <<<"$viewing_key_response" | cut -d'\' -f 6 | cut -c2-)"
    assert_eq "$viewing_key_response" "success"

    # try to use the new key - should fail
    log 'querying balance for "a" with new viewing key'
    balance_query_a='{"balance":{"address":"'"${ADDRESS[a]}"'","key":"'"$vk2_a"'"}}'
    result="$(compute_query "$contract_addr" "$balance_query_a")"
    assert_eq "$result" "$expected_error"

    # try to use the old key - should succeed
    log 'querying balance for "a" with old viewing key'
    balance_query_a='{"balance":{"address":"'"${ADDRESS[a]}"'","key":"'"${VK[a]}"'"}}'
    result="$(compute_query "$contract_addr" "$balance_query_a")"
    if ! silent jq -e '.balance.amount | tonumber' <<<"$result"; then
        log "Balance query returned unexpected response: ${result@Q}"
        return 1
    fi
}

function test_permit() {
    set -e
    local contract_addr="$1"

    log_test_header

    # common variables
    local result
    local tx_hash

    # fail due to token not in permit
    secretcli keys delete banana -yf || true
    secretcli keys add banana
    local wrong_contract=$(secretcli keys show -a banana)

    local permit
    permit='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$wrong_contract"'"],"permissions":["balance"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit_query
    local expected_error="Error: query result: Generic error: Permit doesn't apply to token \"$contract_addr\", allowed tokens: [\"$wrong_contract\"]"
    for key in "${KEY[@]}"; do
        log "permit querying balance for \"$key\" with wrong permit for that contract"
        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$permit"') --from '$key'")
        permit_query='{"with_permit":{"query":{"balance":{}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$wrong_contract"'"],"permissions":["balance"]},"signature":'"$permit"'}}}'
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_error"
    done

    # fail due to revoked permit
    local permit
    permit='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"to_be_revoked","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit_query
    local expected_error
    for key in "${KEY[@]}"; do
        log "permit querying balance for \"$key\" with a revoked permit"
        tx_hash="$(compute_execute "$contract_addr" '{"revoke_permit":{"permit_name":"to_be_revoked"}}' ${FROM[$key]} --gas 250000)"
        wait_for_compute_tx "$tx_hash" "waiting for revoke_permit from \"$key\" to process"

        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$permit"') --from '$key'")
        permit_query='{"with_permit":{"query":{"balance":{}},"permit":{"params":{"permit_name":"to_be_revoked","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]},"signature":'"$permit"'}}}'
        expected_error="Error: query result: Generic error: Permit \"to_be_revoked\" was revoked by account \"${ADDRESS[$key]}"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_error"
    done

    # fail due to params not matching params that were signed on
    local permit
    permit='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit_query
    local expected_error
    for key in "${KEY[@]}"; do
        log "permit querying balance for \"$key\" with params not matching permit"
        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$permit"') --from '$key'")
        permit_query='{"with_permit":{"query":{"balance":{}},"permit":{"params":{"permit_name":"test","chain_id":"not_blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]},"signature":'"$permit"'}}}'
        expected_error="Error: query result: Generic error: Failed to verify signatures for the given permit"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_error"
    done

    # fail balance query due to no balance permission
    local permit_conf
    permit_conf='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$contract_addr"'"],"permissions":["history"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit
    local permit_query
    local expected_error
    for key in "${KEY[@]}"; do
        log "permit querying balance for \"$key\" without the right permission"
        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$permit_conf"') --from '$key'")
        permit_query='{"with_permit":{"query":{"balance":{}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["history"]},"signature":'"$permit"'}}}'
        expected_error="Error: query result: Generic error: No permission to query balance, got permissions [History]"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_error"
    done

    # fail history query due to no history permission
    local permit_conf
    permit_conf='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit
    local permit_query
    local expected_error
    for key in "${KEY[@]}"; do
        log "permit querying history for \"$key\" without the right permission"
        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$permit_conf"') --from '$key'")

        permit_query='{"with_permit":{"query":{"transfer_history":{"page_size":10}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]},"signature":'"$permit"'}}}'
        expected_error="Error: query result: Generic error: No permission to query history, got permissions [Balance]"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_error"

        permit_query='{"with_permit":{"query":{"transaction_history":{"page_size":10}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]},"signature":'"$permit"'}}}'
        expected_error="Error: query result: Generic error: No permission to query history, got permissions [Balance]"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_error"
    done

    # fail allowance query due to no allowance permission
    local permit_conf
    permit_conf='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$contract_addr"'"],"permissions":["history"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit
    local permit_query
    local expected_error
    for key in "${KEY[@]}"; do
        log "permit querying allowance for \"$key\" without the right permission"
        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$permit_conf"') --from '$key'")
        permit_query='{"with_permit":{"query":{"allowance":{"owner":"'"${ADDRESS[$key]}"'","spender":"'"${ADDRESS[$key]}"'"}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["history"]},"signature":'"$permit"'}}}'
        expected_error="Error: query result: Generic error: No permission to query allowance, got permissions [History]"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_error"
    done

    # fail allowance query due to no permit signer not owner or spender
    local permit
    wrong_permit='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$contract_addr"'"],"permissions":["allowance"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit_query
    local expected_error
    log "permit querying allowance without signer being the owner or spender"
    permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$wrong_permit"') --from a")
    permit_query='{"with_permit":{"query":{"allowance":{"owner":"'"$wrong_contract"'","spender":"'"$wrong_contract"'"}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["allowance"]},"signature":'"$permit"'}}}'
    expected_error="Error: query result: Generic error: Cannot query allowance. Requires permit for either owner \"$wrong_contract\" or spender \"$wrong_contract\", got permit for \"${ADDRESS[a]}"
    result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
    assert_eq "$result" "$expected_error"

    # succeed balance query
    local permit
    local good_permit
    good_permit='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit_query
    local expected_output
    for key in "${KEY[@]}"; do
        log "permit querying balance for \"$key\""
        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$good_permit"') --from '$key'")
        permit_query='{"with_permit":{"query":{"balance":{}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["balance"]},"signature":'"$permit"'}}}'
        expected_output="{\"balance\":{\"amount\":\"0\"}}"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_output"
    done

    # succeed history queries
    local permit
    local good_permit
    good_permit='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$contract_addr"'"],"permissions":["history"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit_query
    local expected_output
    for key in "${KEY[@]}"; do
        log "permit querying history for \"$key\""
        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$good_permit"') --from '$key'")

        permit_query='{"with_permit":{"query":{"transfer_history":{"page_size":10}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["history"]},"signature":'"$permit"'}}}'
        expected_output="{\"transfer_history\":{\"txs\":[],\"total\":0}}"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_output"

        permit_query='{"with_permit":{"query":{"transaction_history":{"page_size":10}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["history"]},"signature":'"$permit"'}}}'
        expected_output="{\"transaction_history\":{\"txs\":[],\"total\":0}}"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_output"
    done

    # succeed allowance query
    local permit
    local good_permit
    good_permit='{"account_number":"0","sequence":"0","chain_id":"blabla","msgs":[{"type":"query_permit","value":{"permit_name":"test","allowed_tokens":["'"$contract_addr"'"],"permissions":["allowance"]}}],"fee":{"amount":[{"denom":"uscrt","amount":"0"}],"gas":"1"},"memo":""}'
    local permit_query
    local expected_output
    for key in "${KEY[@]}"; do
        log "permit querying history for \"$key\""
        permit=$(docker exec secretdev bash -c "/usr/bin/secretd tx sign-doc <(echo '"$good_permit"') --from '$key'")

        permit_query='{"with_permit":{"query":{"allowance":{"owner":"'"${ADDRESS[$key]}"'","spender":"'"${ADDRESS[$key]}"'"}},"permit":{"params":{"permit_name":"test","chain_id":"blabla","allowed_tokens":["'"$contract_addr"'"],"permissions":["allowance"]},"signature":'"$permit"'}}}'
        expected_output="{\"allowance\":{\"spender\":\"${ADDRESS[$key]}\",\"owner\":\"${ADDRESS[$key]}\",\"allowance\":\"0\",\"expiration\":null}}"
        result="$(compute_query "$contract_addr" "$permit_query" 2>&1 | sed 's/\\//g' || true)"
        assert_eq "$result" "$expected_output"
    done
}

function test_deposit() {
    set -e
    local contract_addr="$1"

    log_test_header

    local tx_hash

    local -A deposits=([a]=1000000 [b]=2000000 [c]=3000000 [d]=4000000)
    local tx_hash
    local native_tx
    local timestamp
    local block_height
    for key in "${KEY[@]}"; do
        tx_hash="$(deposit "$contract_addr" "$key" "${deposits[$key]}")"
        native_tx="$(secretcli q tx "$tx_hash")"

        timestamp="$(unix_time_of_tx "$native_tx")"
        block_height="$(jq -r '.height' <<<"$native_tx")"
        quiet check_latest_tx_history_deposit "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" "${deposits[$key]}" "$timestamp" "$block_height"
    done

    # Query the balances of the accounts and make sure they have the right balances.
    for key in "${KEY[@]}"; do
        assert_eq "$(get_balance "$contract_addr" "$key")" "${deposits[$key]}"
    done

    # Try to overdraft
    local redeem_message
    local overdraft
    local redeem_response
    for key in "${KEY[@]}"; do
        overdraft="$((deposits[$key] + 1))"
        redeem_message='{"redeem":{"amount":"'"$overdraft"'","denom":"uscrt"}}'
        tx_hash="$(compute_execute "$contract_addr" "$redeem_message" ${FROM[$key]} --gas 150000)"
        # Notice the `!` before the command - it is EXPECTED to fail.
        ! redeem_response="$(wait_for_compute_tx "$tx_hash" "waiting for overdraft from \"$key\" to process")"
        log "trying to overdraft from \"$key\" was rejected"
        assert_eq \
            "$(extract_exec_error "$redeem_response" "error: ")" \
            "insufficient funds to redeem: balance=${deposits[$key]}, required=$overdraft"
    done

    # Withdraw Everything
    local tx_hash
    local native_tx
    local timestamp
    local block_height
    for key in "${KEY[@]}"; do
        tx_hash="$(redeem "$contract_addr" "$key" "${deposits[$key]}")"
        native_tx="$(secretcli q tx "$tx_hash")"
        timestamp="$(unix_time_of_tx "$native_tx")"
        block_height="$(jq -r '.height' <<<"$native_tx")"
        quiet check_latest_tx_history_redeem "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" "${deposits[$key]}" "$timestamp" "$block_height"
    done

    # Check the balances again. They should all be empty
    for key in "${KEY[@]}"; do
        assert_eq "$(get_balance "$contract_addr" "$key")" 0
    done
}

function unix_time_of_tx() {
    set -e
    local tx="$1"

    date -d "$(jq -r '.timestamp' <<<"$tx")" '+%s'
}

function get_transfer_history() {
    set -e
    local contract_addr="$1"
    local account="$2"
    local viewing_key="$3"
    local page_size="$4"
    local page="$5"

    local transfer_history_query
    local transfer_history_response
    transfer_history_query='{"transfer_history":{"address":"'"$account"'","key":"'"$viewing_key"'","page_size":'"$page_size"',"page":'"$page"'}}'
    transfer_history_response="$(compute_query "$contract_addr" "$transfer_history_query")"
    log "$transfer_history_response"
    # There's no good way of tracking the exact expected value,
    # so we just check that the `total` field is a number
    quiet jq -e '.transfer_history.total | numbers' <<<"$transfer_history_response"
    jq -r '.transfer_history.txs' <<<"$transfer_history_response"
}

# This function checks that the latest tx history for the account matches
# the expected parameters.
# The id of the tx is printed out.
function check_latest_transfer_history() {
    set -e
    local contract_addr="$1"
    local account="$2"
    local viewing_key="$3"
    local sender="$4"
    local from="$5"
    local receiver="$6"
    local amount="$7"
    local block_time="$8"
    local block_height="$9"

    local txs
    local tx

    txs="$(get_transfer_history "$contract_addr" "$account" "$viewing_key" 1 0)"
    silent jq -e 'length == 1' <<<"$txs" # just make sure we're not getting a weird response
    tx="$(jq -r '.[0]' <<<"$txs")"
    assert_eq "$(jq -r '.sender' <<<"$tx")" "$sender"
    assert_eq "$(jq -r '.from' <<<"$tx")" "$from"
    assert_eq "$(jq -r '.receiver' <<<"$tx")" "$receiver"
    assert_eq "$(jq -r '.coins.amount' <<<"$tx")" "$amount"
    assert_eq "$(jq -r '.coins.denom' <<<"$tx")" 'SSCRT'
    assert_eq "$(jq -r '.block_time' <<<"$tx")" "$block_time"
    assert_eq "$(jq -r '.block_height' <<<"$tx")" "$block_height"

    jq -r '.id' <<<"$tx"
}

function get_transaction_history() {
    set -e
    local contract_addr="$1"
    local account="$2"
    local viewing_key="$3"
    local page_size="$4"
    local page="$5"

    local transaction_history_query
    local transaction_history_response
    transaction_history_query='{"transaction_history":{"address":"'"$account"'","key":"'"$viewing_key"'","page_size":'"$page_size"',"page":'"$page"'}}'
    transaction_history_response="$(compute_query "$contract_addr" "$transaction_history_query")"
    log "$transaction_history_response"
    # There's no good way of tracking the exact expected value,
    # so we just check that the `total` field is a number
    quiet jq -e '.transaction_history.total | numbers' <<<"$transaction_history_response"
    jq -r '.transaction_history.txs' <<<"$transaction_history_response"
}

function check_latest_tx_history_transfer() {
    set -e
    local contract_addr="$1"
    local account="$2"
    local viewing_key="$3"
    local sender="$4"
    local from="$5"
    local recipient="$6"
    local amount="$7"
    local block_time="$8"
    local block_height="$9"

    local txs
    local tx

    txs="$(get_transaction_history "$contract_addr" "$account" "$viewing_key" 1 0)"
    silent jq -e 'length == 1' <<<"$txs" # just make sure we're not getting a weird response
    tx="$(jq -r '.[0]' <<<"$txs")"
    assert_eq "$(jq -r '.action.transfer.sender' <<<"$tx")" "$sender"
    assert_eq "$(jq -r '.action.transfer.from' <<<"$tx")" "$from"
    assert_eq "$(jq -r '.action.transfer.recipient' <<<"$tx")" "$recipient"
    assert_eq "$(jq -r '.coins.amount' <<<"$tx")" "$amount"
    assert_eq "$(jq -r '.coins.denom' <<<"$tx")" 'SSCRT'
    assert_eq "$(jq -r '.block_time' <<<"$tx")" "$block_time"
    assert_eq "$(jq -r '.block_height' <<<"$tx")" "$block_height"

    jq -r '.id' <<<"$tx"
}

function check_latest_tx_history_mint() {
    set -e
    local contract_addr="$1"
    local account="$2"
    local viewing_key="$3"
    local minter="$4"
    local recipient="$5"
    local amount="$6"
    local block_time="$7"
    local block_height="$8"

    local txs
    local tx

    txs="$(get_transaction_history "$contract_addr" "$account" "$viewing_key" 1 0)"
    silent jq -e 'length == 1' <<<"$txs" # just make sure we're not getting a weird response
    tx="$(jq -r '.[0]' <<<"$txs")"
    assert_eq "$(jq -r '.action.mint.minter' <<<"$tx")" "$minter"
    assert_eq "$(jq -r '.action.mint.recipient' <<<"$tx")" "$recipient"
    assert_eq "$(jq -r '.coins.amount' <<<"$tx")" "$amount"
    assert_eq "$(jq -r '.coins.denom' <<<"$tx")" 'SSCRT'
    assert_eq "$(jq -r '.block_time' <<<"$tx")" "$block_time"
    assert_eq "$(jq -r '.block_height' <<<"$tx")" "$block_height"

    jq -r '.id' <<<"$tx"
}

function check_latest_tx_history_burn() {
    set -e
    local contract_addr="$1"
    local account="$2"
    local viewing_key="$3"
    local burner="$4"
    local owner="$5"
    local amount="$6"
    local block_time="$7"
    local block_height="$8"

    local txs
    local tx

    txs="$(get_transaction_history "$contract_addr" "$account" "$viewing_key" 1 0)"
    silent jq -e 'length == 1' <<<"$txs" # just make sure we're not getting a weird response
    tx="$(jq -r '.[0]' <<<"$txs")"
    assert_eq "$(jq -r '.action.burn.burner' <<<"$tx")" "$burner"
    assert_eq "$(jq -r '.action.burn.owner' <<<"$tx")" "$owner"
    assert_eq "$(jq -r '.coins.amount' <<<"$tx")" "$amount"
    assert_eq "$(jq -r '.coins.denom' <<<"$tx")" 'SSCRT'
    assert_eq "$(jq -r '.block_time' <<<"$tx")" "$block_time"
    assert_eq "$(jq -r '.block_height' <<<"$tx")" "$block_height"

    jq -r '.id' <<<"$tx"
}

function check_latest_tx_history_deposit() {
    set -e
    local contract_addr="$1"
    local account="$2"
    local viewing_key="$3"
    local amount="$4"
    local block_time="$5"
    local block_height="$6"

    local txs
    local tx

    txs="$(get_transaction_history "$contract_addr" "$account" "$viewing_key" 1 0)"
    silent jq -e 'length == 1' <<<"$txs" # just make sure we're not getting a weird response
    tx="$(jq -r '.[0]' <<<"$txs")"
    quiet jq -e '.action.deposit | objects' <<<"$tx"
    assert_eq "$(jq -r '.coins.amount' <<<"$tx")" "$amount"
    assert_eq "$(jq -r '.coins.denom' <<<"$tx")" 'uscrt'
    assert_eq "$(jq -r '.block_time' <<<"$tx")" "$block_time"
    assert_eq "$(jq -r '.block_height' <<<"$tx")" "$block_height"

    jq -r '.id' <<<"$tx"
}

function check_latest_tx_history_redeem() {
    set -e
    local contract_addr="$1"
    local account="$2"
    local viewing_key="$3"
    local amount="$4"
    local block_time="$5"
    local block_height="$6"

    local txs
    local tx

    txs="$(get_transaction_history "$contract_addr" "$account" "$viewing_key" 1 0)"
    silent jq -e 'length == 1' <<<"$txs" # just make sure we're not getting a weird response
    tx="$(jq -r '.[0]' <<<"$txs")"
    quiet jq -e '.action.redeem | objects' <<<"$tx"
    assert_eq "$(jq -r '.coins.amount' <<<"$tx")" "$amount"
    assert_eq "$(jq -r '.coins.denom' <<<"$tx")" 'SSCRT'
    assert_eq "$(jq -r '.block_time' <<<"$tx")" "$block_time"
    assert_eq "$(jq -r '.block_height' <<<"$tx")" "$block_height"

    jq -r '.id' <<<"$tx"
}

function test_transfer() {
    set -e
    local contract_addr="$1"

    log_test_header

    local tx_hash

    # Check "a" and "b" don't have any funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 0
    assert_eq "$(get_balance "$contract_addr" 'b')" 0

    # Deposit to "a"
    quiet deposit "$contract_addr" 'a' 1000000

    # Try to transfer more than "a" has
    log 'transferring funds from "a" to "b", but more than "a" has'
    local transfer_message='{"transfer":{"recipient":"'"${ADDRESS[b]}"'","amount":"1000001"}}'
    local transfer_response
    tx_hash="$(compute_execute "$contract_addr" "$transfer_message" ${FROM[a]} --gas 150000)"
    # Notice the `!` before the command - it is EXPECTED to fail.
    ! transfer_response="$(wait_for_compute_tx "$tx_hash" 'waiting for transfer from "a" to "b" to process')"
    log "trying to overdraft from \"a\" to transfer to \"b\" was rejected"
    assert_eq "$(extract_exec_error "$transfer_response" "error: ")" "insufficient funds: balance=1000000, required=1000001"

    # Check both a and b, that their last transaction is not for 1000001 uscrt
    local txs
    for key in a b; do
        log "querying the transfer history of \"$key\""
        txs="$(get_transfer_history "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" 1 0)"
        silent jq -e 'length <= 1' <<<"$txs" # just make sure we're not getting a weird response
        if silent jq -e 'length == 1' <<<"$txs"; then
            assert_ne "$(jq -r '.[0].coins.amount' <<<"$txs")" 1000001
        fi
    done

    # Transfer from "a" to "b"
    log 'transferring funds from "a" to "b"'
    local transfer_message='{"transfer":{"recipient":"'"${ADDRESS[b]}"'","amount":"400000"}}'
    local transfer_response
    tx_hash="$(compute_execute "$contract_addr" "$transfer_message" ${FROM[a]} --gas 200000)"
    transfer_response="$(data_of wait_for_compute_tx "$tx_hash" 'waiting for transfer from "a" to "b" to process')"
    assert_eq "$transfer_response" "$(pad_space '{"transfer":{"status":"success"}}' | sed 's/ //g')"

    local native_tx
    native_tx="$(secretcli q tx "$tx_hash")"
    local timestamp
    timestamp="$(unix_time_of_tx "$native_tx")"
    local block_height
    block_height="$(jq -r '.height' <<<"$native_tx")"

    # Check for both "a" and "b" that they recorded the transfer
    local -A tx_ids
    local tx_id
    for key in a b; do
        log "querying the transfer history of \"$key\""
        tx_ids[$key]="$(
            check_latest_transfer_history "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" \
                "${ADDRESS[a]}" "${ADDRESS[a]}" "${ADDRESS[b]}" 400000 "$timestamp" "$block_height"
        )"
        tx_id="$(
            check_latest_tx_history_transfer "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" \
                "${ADDRESS[a]}" "${ADDRESS[a]}" "${ADDRESS[b]}" 400000 "$timestamp" "$block_height"
        )"
        assert_eq "$tx_id" "${tx_ids[$key]}"
    done

    assert_eq "${tx_ids[a]}" "${tx_ids[b]}"
    log 'The transfer was recorded correctly in the transaction history'

    # Check that "a" has fewer funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 600000

    # Check that "b" has the funds that "a" deposited
    assert_eq "$(get_balance "$contract_addr" 'b')" 400000

    # Redeem both accounts
    redeem "$contract_addr" a 600000
    redeem "$contract_addr" b 400000
    # Send the funds back
    quiet secretcli tx bank send b "${ADDRESS[a]}" 400000uscrt -y -b block
}

RECEIVER_ADDRESS=''

function create_receiver_contract() {
    set -e
    local init_msg

    if [[ "$RECEIVER_ADDRESS" != '' ]]; then
        log 'Receiver contract already exists'
        echo "$RECEIVER_ADDRESS"
        return 0
    fi

    init_msg='{"count":0}'
    RECEIVER_ADDRESS="$(create_contract 'tests/example-receiver' "$init_msg")"

    log "uploaded receiver contract to $RECEIVER_ADDRESS"
    echo "$RECEIVER_ADDRESS"
}

# This function exists so that we can reset the state as much as possible between different tests
function redeem_receiver() {
    set -e
    local receiver_addr="$1"
    local snip20_addr="$2"
    local to_addr="$3"
    local amount="$4"

    local tx_hash
    local redeem_tx
    local transfer_attributes

    log 'fetching snip20 hash'
    local snip20_hash
    snip20_hash="$(secretcli query compute contract-hash "$snip20_addr")"

    local redeem_message='{"redeem":{"addr":"'"$snip20_addr"'","hash":"'"${snip20_hash:2}"'","to":"'"$to_addr"'","amount":"'"$amount"'","denom":"uscrt"}}'
    tx_hash="$(compute_execute "$receiver_addr" "$redeem_message" ${FROM[a]} --gas 300000)"
    redeem_tx="$(wait_for_tx "$tx_hash" "waiting for redeem from receiver at \"$receiver_addr\" to process")"
    # log "$redeem_tx"
    transfer_attributes="$(jq -r '.logs[0].events[] | select(.type == "transfer") | .attributes' <<<"$redeem_tx")"
    assert_eq "$(jq -r '.[] | select(.key == "recipient") | .value' <<<"$transfer_attributes")" "$receiver_addr"$'\n'"$to_addr"
    assert_eq "$(jq -r '.[] | select(.key == "amount") | .value' <<<"$transfer_attributes")" "${amount}uscrt"$'\n'"${amount}uscrt"
    log "redeem response for \"$receiver_addr\" returned ${amount}uscrt"
}

function register_receiver() {
    set -e
    local receiver_addr="$1"
    local snip20_addr="$2"

    local tx_hash

    log 'fetching snip20 hash'
    local snip20_hash
    snip20_hash="$(secretcli query compute contract-hash "$snip20_addr")"

    log 'registering with snip20'
    local register_message='{"register":{"reg_addr":"'"$snip20_addr"'","reg_hash":"'"${snip20_hash:2}"'"}}'
    tx_hash="$(compute_execute "$receiver_addr" "$register_message" ${FROM[a]} --gas 300000)"

    # we throw away the output since we know it's empty
    local register_tx
    register_tx="$(wait_for_compute_tx "$tx_hash" 'Waiting for receiver registration')"

    assert_eq \
        "$(jq -r '.output_logs[] | select(.type == "wasm") | .attributes[] | select(.key == "register_status                                                                                                                                                                                                                                                 ") | .value' <<<"$register_tx")" \
        'success'
    log 'receiver registered successfully'
}

function test_send() {
    set -e
    local contract_addr="$1"
    local skip_register_receiver="$2"

    log_test_header

    local receiver_addr
    receiver_addr="$(create_receiver_contract)"
    local receiver_hash
    receiver_hash="$(secretcli q compute contract-hash $receiver_addr | sed 's/^0x//')"

    if [ "$skip_register_receiver" != "skip-register" ]; then
        register_receiver "$receiver_addr" "$contract_addr"
    fi

    local tx_hash

    # Check "a" and "b" don't have any funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 0
    assert_eq "$(get_balance "$contract_addr" 'b')" 0

    # Deposit to "a"
    quiet deposit "$contract_addr" 'a' 1000000

    # Try to send more than "a" has
    log 'sending funds from "a" to "b", but more than "a" has'
    local send_message
    send_message='{"send":{"recipient":"'"${ADDRESS[b]}"'","amount":"1000001"}}'
    local send_response
    tx_hash="$(compute_execute "$contract_addr" "$send_message" ${FROM[a]} --gas 150000)"
    # Notice the `!` before the command - it is EXPECTED to fail.
    ! send_response="$(wait_for_compute_tx "$tx_hash" 'waiting for send from "a" to "b" to process')"
    log "trying to overdraft from \"a\" to send to \"b\" was rejected"

    assert_eq "$(extract_exec_error "$send_response" "error: ")" "insufficient funds: balance=1000000, required=1000001"

    # Check both a and b, that their last transaction is not for 1000001 uscrt
    local txs
    for key in a b; do
        log "querying the transfer history of \"$key\""
        txs="$(get_transfer_history "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" 1 0)"
        silent jq -e 'length <= 1' <<<"$txs" # just make sure we're not getting a weird response
        if silent jq -e 'length == 1' <<<"$txs"; then
            assert_ne "$(jq -r '.[0].coins.amount' <<<"$txs")" 1000001
        fi
    done

    # Query receiver state before Send
    local receiver_state
    local receiver_state_query='{"get_count":{}}'
    receiver_state="$(compute_query "$receiver_addr" "$receiver_state_query")"
    local original_count
    original_count="$(jq -r '.count' <<<"$receiver_state")"

    # Send from "a" to the receiver with message to the Receiver
    log 'sending funds from "a" to the Receiver, with message to the Receiver'
    local receiver_msg='{"increment":{}}'
    receiver_msg="$(base64 <<<"$receiver_msg")"

    if [ "$skip_register_receiver" = "skip-register" ]; then
        send_message='{"send":{"recipient":"'"$receiver_addr"'","recipient_code_hash":"'"$receiver_hash"'","amount":"400000","msg":"'$receiver_msg'"}}'
    else
        send_message='{"send":{"recipient":"'"$receiver_addr"'","amount":"400000","msg":"'$receiver_msg'"}}'
    fi

    local send_response

    tx_hash="$(compute_execute "$contract_addr" "$send_message" ${FROM[a]} --gas 300000)"
    send_response="$(wait_for_compute_tx "$tx_hash" 'waiting for send from "a" to the Receiver to process')"
    assert_eq \
        "$(jq -r '.output_logs[0].attributes[] | select(.key == "count") | .value' <<<"$send_response")" \
        "$((original_count + 1))"
    log 'received send response'

    local native_tx
    native_tx="$(secretcli q tx "$tx_hash")"
    local timestamp
    timestamp="$(unix_time_of_tx "$native_tx")"
    local block_height
    block_height="$(jq -r '.height' <<<"$native_tx")"

    # Check that the receiver got the message
    log 'checking whether state was updated in the receiver'
    receiver_state_query='{"get_count":{}}'
    receiver_state="$(compute_query "$receiver_addr" "$receiver_state_query")"
    local new_count
    new_count="$(jq -r '.count' <<<"$receiver_state")"
    assert_eq "$((original_count + 1))" "$new_count"
    log 'receiver contract received the message'

    # Check that "a" recorded the transfer
    log 'querying the transfer history of "a"'
    local tx_id1
    local tx_id2
    tx_id1="$(check_latest_transfer_history "$contract_addr" "${ADDRESS[a]}" "${VK[a]}" \
        "${ADDRESS[a]}" "${ADDRESS[a]}" "$receiver_addr" 400000 "$timestamp" "$block_height")"
    tx_id2="$(check_latest_tx_history_transfer "$contract_addr" "${ADDRESS[a]}" "${VK[a]}" \
        "${ADDRESS[a]}" "${ADDRESS[a]}" "$receiver_addr" 400000 "$timestamp" "$block_height")"
    assert_eq "$tx_id1" "$tx_id2"

    # Check that "a" has fewer funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 600000

    # Test that send callback failure also denies the transfer
    log 'sending funds from "a" to the Receiver, with a "Fail" message to the Receiver'
    receiver_msg='{"fail":{}}'
    receiver_msg="$(base64 <<<"$receiver_msg")"

    if [ "$skip_register_receiver" = "skip-register" ]; then
        send_message='{"send":{"recipient":"'"$receiver_addr"'","recipient_code_hash":"'"$receiver_hash"'","amount":"400000","msg":"'$receiver_msg'"}}'
    else
        send_message='{"send":{"recipient":"'"$receiver_addr"'","amount":"400000","msg":"'$receiver_msg'"}}'
    fi

    tx_hash="$(compute_execute "$contract_addr" "$send_message" ${FROM[a]} --gas 300000)"
    # Notice the `!` before the command - it is EXPECTED to fail.
    ! send_response="$(wait_for_compute_tx "$tx_hash" 'waiting for send from "a" to the Receiver to process')"
    assert_eq "$(extract_exec_error "$send_response" "error: ")" 'intentional failure' # This comes from the receiver contract

    # Check that "a" does not have fewer funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 600000 # This is the same balance as before

    log 'a failure in the callback caused the transfer to roll back, as expected'

    # redeem both accounts
    redeem "$contract_addr" 'a' 600000
    redeem_receiver "$receiver_addr" "$contract_addr" "${ADDRESS[a]}" 400000
}

function set_minters() {
    #    set -e
    local minters

    if (( $# == 0 )); then
        minters='[]'
    else
        minters='["'

        for minter in "${@:1:$(($# - 1))}"; do
            minters="${minters}${minter}"'","'
        done

        minters="${minters}${*: -1}"'"]'
    fi

    log "$minters"

    local set_minters_message='{"set_minters":{"minters":'"$minters"'}}'
    local tx_hash
    local response
    tx_hash="$(compute_execute "$contract_addr" "$set_minters_message" ${FROM[a]} --gas 150000)"
    response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for minter set to update")"
    assert_eq "$response" "$(pad_space '{"set_minters":{"status":"success"}}' | sed 's/ //g')"
    log "set the minters to these addresses: $*"
}

# This test also tests TokenInfo
function test_burn() {
    set -e
    local contract_addr="$1"

    log_test_header

    local token_info_response
    local burn_response
    local tx_hash

    set_minters "${ADDRESS[b]}"

    # minting from the wrong account should fail
    set +e; tx_hash="$(mint "$contract_addr" 'a' "${ADDRESS[a]}" 1000000)"
    local _res="$?"; set -e;
    if (( _res != 0 )); then
        assert_eq "$(extract_exec_error "$(secretcli query compute tx "$tx_hash")" "error: ")" 'Minting is allowed to minter accounts only'
        log 'minting from the wrong account failed as expected'
    else
        log 'minting was allowed from a non-minter address!'
        return 1
    fi

    # Mint to a using b
    local native_tx
    local timestamp
    local block_height
    tx_hash="$(mint "$contract_addr" 'b' "${ADDRESS[a]}" 1000000)"
    native_tx="$(secretcli q tx "$tx_hash")"
    timestamp="$(unix_time_of_tx "$native_tx")"
    block_height="$(jq -r '.height' <<<"$native_tx")"
    check_latest_tx_history_mint "$contract_addr" "${ADDRESS[a]}" "${VK[a]}" \
        "${ADDRESS[b]}" "${ADDRESS[a]}" 1000000 "$timestamp" "$block_height"

    # Check total supply
    token_info_response="$(get_token_info "$contract_addr")"
    log 'token info response was' "$token_info_response"
    assert_eq "$(jq -r '.token_info.total_supply' <<<"$token_info_response")" 1000000

    # Try to over-burn
    local burn_message='{"burn":{"amount":"10000000"}}' # 110%
    local burn_response
    tx_hash="$(compute_execute "$contract_addr" "$burn_message" ${FROM[a]} --gas 150000)"
    ! burn_response="$(wait_for_compute_tx "$tx_hash" 'waiting for burn for "a" to process')"
    assert_eq "$(extract_exec_error "$burn_response" "error: ")" 'insufficient funds to burn: balance=1000000, required=10000000'

    # Check "a" balance - should not have changes
    assert_eq "$(get_balance "$contract_addr" 'a')" 1000000

    # Check total supply
    token_info_response="$(get_token_info "$contract_addr")"
    log 'token info response was' "$token_info_response"
    assert_eq "$(jq -r '.token_info.total_supply' <<<"$token_info_response")" 1000000

    # Try to burn
    quiet burn "$contract_addr" 'a' 100000 # 10%

    # Check "a" balance
    assert_eq "$(get_balance "$contract_addr" 'a')" 900000

    # Check total supply
    token_info_response="$(get_token_info "$contract_addr")"
    log 'token info response was' "$token_info_response"
    assert_eq "$(jq -r '.token_info.total_supply' <<<"$token_info_response")" 900000

    # Burn the rest of the balance
    tx_hash="$(burn "$contract_addr" 'a' 900000)"
    log "the tx_hash is $tx_hash"
    native_tx="$(secretcli q tx "$tx_hash")"
    timestamp="$(unix_time_of_tx "$native_tx")"
    block_height="$(jq -r '.height' <<<"$native_tx")"
    check_latest_tx_history_burn "$contract_addr" "${ADDRESS[a]}" "${VK[a]}" \
        "${ADDRESS[a]}" "${ADDRESS[a]}" 900000 "$timestamp" "$block_height"

    token_info_response="$(get_token_info "$contract_addr")"
    log 'token info response was' "$token_info_response"
    assert_eq "$(jq -r '.token_info.total_supply' <<<"$token_info_response")" 0
}

function test_transfer_from() {
    set -e
    local contract_addr="$1"

    log_test_header

    local tx_hash

    # Check "a", "b", and "c" don't have any funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 0
    assert_eq "$(get_balance "$contract_addr" 'b')" 0
    assert_eq "$(get_balance "$contract_addr" 'c')" 0

    # Check that the allowance given to "b" by "a" is zero
    assert_eq "$(get_allowance "$contract_addr" 'a' 'b')" 0

    # Deposit to "a"
    quiet deposit "$contract_addr" 'a' 1000000

    # Make "a" give allowance to "b"
    assert_eq "$(increase_allowance "$contract_addr" 'a' 'b' 1000000)" 1000000
    assert_eq "$(get_allowance "$contract_addr" 'a' 'b')" 1000000

    # Try to transfer from "a", using "b" more than "a" has allowed
    log 'transferring funds from "a" to "c" using "b", but more than "a" has allowed'
    local transfer_message='{"transfer_from":{"owner":"'"${ADDRESS[a]}"'","recipient":"'"${ADDRESS[c]}"'","amount":"1000001"}}'
    local transfer_response
    tx_hash="$(compute_execute "$contract_addr" "$transfer_message" ${FROM[b]} --gas 150000)"
    # Notice the `!` before the command - it is EXPECTED to fail.
    ! transfer_response="$(wait_for_compute_tx "$tx_hash" 'waiting for transfer from "a" to "c" by "b" to process')"
    log "trying to overdraft from \"a\" to transfer to \"c\" using \"b\" was rejected"
    assert_eq "$(extract_exec_error "$transfer_response" "error: ")" "insufficient allowance: allowance=1000000, required=1000001"

    # Check both "a", "b", and "c", that their last transaction is not for 1000001 uscrt
    local txs
    for key in a b c; do
        log "querying the transfer history of \"$key\""
        txs="$(get_transfer_history "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" 1 0)"
        silent jq -e 'length <= 1' <<<"$txs" # just make sure we're not getting a weird response
        if silent jq -e 'length == 1' <<<"$txs"; then
            assert_ne "$(jq -r '.[0].coins.amount' <<<"$txs")" 1000001
        fi
    done

    # Transfer from "a" to "c" using "b"
    log 'transferring funds from "a" to "c" using "b"'
    local transfer_message='{"transfer_from":{"owner":"'"${ADDRESS[a]}"'","recipient":"'"${ADDRESS[c]}"'","amount":"400000"}}'
    local transfer_response
    tx_hash="$(compute_execute "$contract_addr" "$transfer_message" ${FROM[b]} --gas 250000)"
    transfer_response="$(data_of wait_for_compute_tx "$tx_hash" 'waiting for transfer from "a" to "c" by "b" to process')"
    assert_eq "$transfer_response" "$(pad_space '{"transfer_from":{"status":"success"}}' | sed 's/ //g')"

    local native_tx
    native_tx="$(secretcli q tx "$tx_hash")"
    local timestamp
    timestamp="$(unix_time_of_tx "$native_tx")"
    local block_height
    block_height="$(jq -r '.height' <<<"$native_tx")"

    # Check for both "a", "b", and "c" that they recorded the transfer
    local -A tx_ids
    local tx_id
    for key in a b c; do
        log "querying the transfer history of \"$key\""
        tx_ids[$key]="$(
            check_latest_transfer_history "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" \
                "${ADDRESS[b]}" "${ADDRESS[a]}" "${ADDRESS[c]}" 400000 "$timestamp" "$block_height"
        )"
        tx_id="$(
            check_latest_tx_history_transfer "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" \
                "${ADDRESS[b]}" "${ADDRESS[a]}" "${ADDRESS[c]}" 400000 "$timestamp" "$block_height"
        )"
        assert_eq "$tx_id" "${tx_ids[$key]}"
    done

    assert_eq "${tx_ids[a]}" "${tx_ids[b]}"
    assert_eq "${tx_ids[b]}" "${tx_ids[c]}"
    log 'The transfer was recorded correctly in the transaction history'

    # Check that "a" has fewer funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 600000

    # Check that "b" has the same funds still, but less allowance
    assert_eq "$(get_balance "$contract_addr" 'b')" 0
    assert_eq "$(get_allowance "$contract_addr" 'a' 'b')" 600000

    # Check that "c" has the funds that "b" deposited from "a"
    assert_eq "$(get_balance "$contract_addr" 'c')" 400000

    # Redeem both accounts
    redeem "$contract_addr" a 600000
    redeem "$contract_addr" c 400000
    # Reset allowance
    assert_eq "$(decrease_allowance "$contract_addr" 'a' 'b' 600000)" 0
    assert_eq "$(get_allowance "$contract_addr" 'a' 'b')" 0
    # Send the funds back
    quiet secretcli tx bank send c "${ADDRESS[a]}" 400000uscrt -y -b block
}

function test_send_from() {
    set -e
    local contract_addr="$1"
    local skip_register_receiver="$2"

    log_test_header

    local receiver_addr
    receiver_addr="$(create_receiver_contract)"
    local receiver_hash
    receiver_hash="$(secretcli q compute contract-hash $receiver_addr | sed 's/^0x//')"

    if [ "$skip_register_receiver" != "skip-register" ]; then
        register_receiver "$receiver_addr" "$contract_addr"
    fi

    local tx_hash

    # Check "a" and "b" don't have any funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 0
    assert_eq "$(get_balance "$contract_addr" 'b')" 0
    assert_eq "$(get_balance "$contract_addr" 'c')" 0

    # Check that the allowance given to "b" by "a" is zero
    assert_eq "$(get_allowance "$contract_addr" 'a' 'b')" 0

    # Deposit to "a"
    quiet deposit "$contract_addr" 'a' 1000000

    # Make "a" give allowance to "b"
    assert_eq "$(increase_allowance "$contract_addr" 'a' 'b' 1000000)" 1000000
    assert_eq "$(get_allowance "$contract_addr" 'a' 'b')" 1000000

    # TTry to send from "a", using "b" more than "a" has allowed
    log 'sending funds from "a" to "c" using "b", but more than "a" has allowed'
    local send_message='{"send_from":{"owner":"'"${ADDRESS[a]}"'","recipient":"'"${ADDRESS[c]}"'","amount":"1000001"}}'
    local send_response
    tx_hash="$(compute_execute "$contract_addr" "$send_message" ${FROM[b]} --gas 150000)"
    # Notice the `!` before the command - it is EXPECTED to fail.
    ! send_response="$(wait_for_compute_tx "$tx_hash" 'waiting for send from "a" to "c" by "b" to process')"
    log "trying to overdraft from \"a\" to send to \"c\" using \"b\" was rejected"
    assert_eq "$(extract_exec_error "$send_response" "error: ")" "insufficient allowance: allowance=1000000, required=1000001"

    # Check both a and b, that their last transaction is not for 1000001 uscrt
    local txs
    for key in a b c; do
        log "querying the transfer history of \"$key\""
        txs="$(get_transfer_history "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" 1 0)"
        silent jq -e 'length <= 1' <<<"$txs" # just make sure we're not getting a weird response
        if silent jq -e 'length == 1' <<<"$txs"; then
            assert_ne "$(jq -r '.[0].coins.amount' <<<"$txs")" 1000001
        fi
    done

    # Query receiver state before Send
    local receiver_state
    local receiver_state_query='{"get_count":{}}'
    receiver_state="$(compute_query "$receiver_addr" "$receiver_state_query")"
    local original_count
    original_count="$(jq -r '.count' <<<"$receiver_state")"

    # Send from "a", using "b", to the receiver with message to the Receiver
    log 'sending funds from "a", using "b", to the Receiver, with message to the Receiver'
    local receiver_msg='{"increment":{}}'
    receiver_msg="$(base64 <<<"$receiver_msg")"

    local send_message
    if [ "$skip_register_receiver" = "skip-register" ]; then
        send_message='{"send_from":{"owner":"'"${ADDRESS[a]}"'","recipient":"'"$receiver_addr"'","recipient_code_hash":"'"$receiver_hash"'","amount":"400000","msg":"'$receiver_msg'"}}'
    else
        send_message='{"send_from":{"owner":"'"${ADDRESS[a]}"'","recipient":"'"$receiver_addr"'","amount":"400000","msg":"'$receiver_msg'"}}'
    fi

    local send_response
    tx_hash="$(compute_execute "$contract_addr" "$send_message" ${FROM[b]} --gas 302000)"
    send_response="$(wait_for_compute_tx "$tx_hash" 'waiting for send from "a" to the Receiver to process')"
    assert_eq \
        "$(jq -r '.output_logs[0].attributes[] | select(.key == "count") | .value' <<<"$send_response")" \
        "$((original_count + 1))"
    log 'received send response'

    local native_tx
    native_tx="$(secretcli q tx "$tx_hash")"
    local timestamp
    timestamp="$(unix_time_of_tx "$native_tx")"
    local block_height
    block_height="$(jq -r '.height' <<<"$native_tx")"

    # Check that the receiver got the message
    log 'checking whether state was updated in the receiver'
    receiver_state_query='{"get_count":{}}'
    receiver_state="$(compute_query "$receiver_addr" "$receiver_state_query")"
    local new_count
    new_count="$(jq -r '.count' <<<"$receiver_state")"
    assert_eq "$((original_count + 1))" "$new_count"
    log 'receiver contract received the message'

    # Check that "a" recorded the transfer
    local -A tx_ids
    local tx_id
    for key in a b; do
        log "querying the transfer history of \"$key\""
        tx_ids[$key]="$(
            check_latest_transfer_history "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" \
                "${ADDRESS[b]}" "${ADDRESS[a]}" "$receiver_addr" 400000 "$timestamp" "$block_height"
        )"
        tx_id="$(
            check_latest_tx_history_transfer "$contract_addr" "${ADDRESS[$key]}" "${VK[$key]}" \
                "${ADDRESS[b]}" "${ADDRESS[a]}" "$receiver_addr" 400000 "$timestamp" "$block_height"
        )"
        assert_eq "$tx_id" "${tx_ids[$key]}"
    done

    assert_eq "${tx_ids[a]}" "${tx_ids[b]}"
    log 'The transfer was recorded correctly in the transaction history'

    # Check that "a" has fewer funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 600000

    # Check that "b" has the same funds still, but less allowance
    assert_eq "$(get_balance "$contract_addr" 'b')" 0
    assert_eq "$(get_allowance "$contract_addr" 'a' 'b')" 600000

    # Test that send callback failure also denies the transfer
    log 'sending funds from "a", using "b", to the Receiver, with a "Fail" message to the Receiver'
    receiver_msg='{"fail":{}}'
    receiver_msg="$(base64 <<<"$receiver_msg")"

    if [ "$skip_register_receiver" = "skip-register" ]; then
        send_message='{"send_from":{"owner":"'"${ADDRESS[a]}"'","recipient":"'"$receiver_addr"'","recipient_code_hash":"'"$receiver_hash"'","amount":"400000","msg":"'$receiver_msg'"}}'
    else
        send_message='{"send_from":{"owner":"'"${ADDRESS[a]}"'","recipient":"'"$receiver_addr"'","amount":"400000","msg":"'$receiver_msg'"}}'
    fi

    tx_hash="$(compute_execute "$contract_addr" "$send_message" ${FROM[b]} --gas 300000)"
    # Notice the `!` before the command - it is EXPECTED to fail.
    ! send_response="$(wait_for_compute_tx "$tx_hash" 'waiting for send from "a" to the Receiver to process')"
    assert_eq "$(extract_exec_error "$send_response" "error: ")" 'intentional failure' # This comes from the receiver contract

    # Check that "a" does not have fewer funds
    assert_eq "$(get_balance "$contract_addr" 'a')" 600000 # This is the same balance as before

    log 'a failure in the callback caused the transfer to roll back, as expected'

    # redeem both accounts
    redeem "$contract_addr" 'a' 600000
    redeem_receiver "$receiver_addr" "$contract_addr" "${ADDRESS[a]}" 400000
    # Reset allowance
    assert_eq "$(decrease_allowance "$contract_addr" 'a' 'b' 600000)" 0
    assert_eq "$(get_allowance "$contract_addr" 'a' 'b')" 0
}

function main() {
    log '              <####> Starting integration tests <####>'
    log "secretcli version in the docker image is: $(secretcli version)"

    secretcli tx bank send a $(secretcli keys show -a c) 100000000000uscrt -y -b block > /dev/null
    secretcli tx bank send a $(secretcli keys show -a d) 100000000000uscrt -y -b block > /dev/null

    local prng_seed
    prng_seed="$(base64 <<<'enigma-rocks')"
    local init_msg
    init_msg='{"name":"secret-secret","admin":"'"${ADDRESS[a]}"'","symbol":"SSCRT","decimals":6,"initial_balances":[],"prng_seed":"'"$prng_seed"'","config":{"public_total_supply":true,"enable_deposit":true,"enable_redeem":true,"enable_mint":true,"enable_burn":true},"supported_denoms":["uscrt"]}'
    contract_addr="$(create_contract '.' "$init_msg")"

    # To make testing faster, check the logs and try to reuse the deployed contract and VKs from previous runs.
    # Remember to comment out the contract deployment and `test_viewing_key` if you do.
#    local contract_addr='secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg'
#    VK[a]='api_key_U6FcuhP2km6UHtYeFSyaZbggcgMJQAiTMlNWV3X4iXQ='
#    VK[b]='api_key_YoQlmqnOkkEoh81XzFkiZ3z7+ZAJh9kyFXvtaMBhiFU='
#    VK[c]='api_key_/cdkitEbzaHZA41OB6cGcz1XGnQk6LYTAfSBWTOU5aQ='
#    VK[d]='api_key_WQYkuGOco/mSHgtKWG0f7b2UcrSG3s1fIqm1X/wIGDo='

    log "contract address: $contract_addr"

    wait_for_tx "$(tx_of secretcli tx bank send "${ADDRESS[a]}" "${ADDRESS[b]}" 100000000uscrt -y)" "waiting for send to b"
    wait_for_tx "$(tx_of secretcli tx bank send "${ADDRESS[a]}" "${ADDRESS[c]}" 100000000uscrt -y)" "waiting for send to c"
    wait_for_tx "$(tx_of secretcli tx bank send "${ADDRESS[a]}" "${ADDRESS[d]}" 100000000uscrt -y)" "waiting for send to d"

    # This first test also sets the `VK[*]` global variables that are used in the other tests
    test_viewing_key "$contract_addr"
    test_permit "$contract_addr"
    test_deposit "$contract_addr"
    test_transfer "$contract_addr"
    test_send "$contract_addr" register
    test_send "$contract_addr" skip-register
    test_burn "$contract_addr"
    test_transfer_from "$contract_addr"
    test_send_from "$contract_addr" register
    test_send_from "$contract_addr" skip-register

    log 'Tests completed successfully'

    # If everything else worked, return successful status
    return 0
}

main "$@"
