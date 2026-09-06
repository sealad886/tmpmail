# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_tmpmail_global_optspecs
    string join \n inbox= provider= no-provider= json no-color timeout= quiet verbose h/help V/version
end

function __fish_tmpmail_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_tmpmail_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_tmpmail_using_subcommand
    set -l cmd (__fish_tmpmail_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c tmpmail -n "__fish_tmpmail_needs_command" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_needs_command" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_needs_command" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_needs_command" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_needs_command" -l json
complete -c tmpmail -n "__fish_tmpmail_needs_command" -l no-color
complete -c tmpmail -n "__fish_tmpmail_needs_command" -l quiet
complete -c tmpmail -n "__fish_tmpmail_needs_command" -l verbose
complete -c tmpmail -n "__fish_tmpmail_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -s V -l version -d 'Print version'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "message" -d 'Work with messages'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "inbox" -d 'Work with named inboxes'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "attachment" -d 'Work with message attachments'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "provider" -d 'Inspect and configure providers'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "config" -d 'Inspect and update local configuration'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "ls" -d 'Alias for `message list`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "read" -d 'Alias for `message read`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "latest" -d 'Alias for `message latest`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "new" -d 'Alias for `inbox create`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "new-mail" -d 'Alias for `message new`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "unread" -d 'Alias for `message new`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "inboxes" -d 'Alias for `inbox list`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "use" -d 'Alias for `inbox use`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "watch" -d 'Alias for `inbox watch`'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "copy" -d 'Copy an inbox address'
complete -c tmpmail -n "__fish_tmpmail_needs_command" -f -a "doctor" -d 'Diagnose local configuration'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -f -a "list"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -f -a "read"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -f -a "latest"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -f -a "new"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and not __fish_seen_subcommand_from list read latest new ack" -f -a "ack"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l peek
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l html
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l open
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l raw
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l peek
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l html
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l open
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l raw
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l all-inboxes
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l review
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from new" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand message; and __fish_seen_subcommand_from ack" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -f -a "create"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -f -a "list"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -f -a "use"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -f -a "remove"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and not __fish_seen_subcommand_from create list use remove watch" -f -a "watch"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l address -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from use" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l all
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l existing
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inbox; and __fish_seen_subcommand_from watch" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -f -a "list"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and not __fish_seen_subcommand_from list save" -f -a "save"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l force
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand attachment; and __fish_seen_subcommand_from save" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -f -a "list"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -f -a "domains"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -f -a "auth"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and not __fish_seen_subcommand_from list domains auth info" -f -a "info"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from domains" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from domains" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from domains" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from domains" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from domains" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from domains" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from domains" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from domains" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -l token-stdin
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from auth" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from info" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from info" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from info" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from info" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from info" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from info" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from info" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand provider; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -f -a "show"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -f -a "get"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -f -a "set"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and not __fish_seen_subcommand_from show get set unset" -f -a "unset"
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand config; and __fish_seen_subcommand_from unset" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand ls" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l peek
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l html
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l open
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l raw
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand read" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l peek
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l html
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l open
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l raw
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand latest" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l address -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l all-inboxes
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand new-mail" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l all-inboxes
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand unread" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand inboxes" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand use" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l all
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l existing
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand watch" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand copy" -s h -l help -d 'Print help'
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -l inbox -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -l provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -l no-provider -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -l timeout -r
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -l json
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -l no-color
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -l quiet
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -l verbose
complete -c tmpmail -n "__fish_tmpmail_using_subcommand doctor" -s h -l help -d 'Print help'
