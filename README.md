# genetec-ident-remover

Supports multiple commands to list & remove objects from Genetec ClearID.
This has been tested against the Genetec ClearID-API identity enpoints v4.

## Prerequisites:

Create an API integration and download the corresponding *.json file. This includes all necessary information for authentication and endpoints.

## Usage:

For help please take a look at:

```bash
genetec-ident-remover help

Usage: genetec-ident-remover <COMMAND>

Commands:
  list-inactive-identities
  purge-inactive-identities
  purge-pictures
  help                       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

```

### Listing Identities

List all inactive identities which have not been modified within the last 90 days from a Genetec ClearID instance.

```bash
genetec-ident-remover list-inactive-identities -k <keyfile_name>.json

```

### Deleting Identities

Removes all inactive Identities which have not been modified within the last 90 days from a Genetec ClearID instance.

```bash
genetec-ident-remover purge-inactive-identities -k <keyfile_name>.json

```

### Deleting Pictures

Removes all Pictures from all Identities in a Genetec ClearID instance.

```bash
genetec-ident-remover purge-pictures -k <keyfile_name>.json --delete

```

## Configuration:

Some other Configuration options are available, to list all options please consult the help of the needed subcommand, e.g.:

```bash
genetec-ident-remover purge-inactive-identities help

Options:
  -k <KEYFILE>
  -i, --inactive-days <INACTIVE_DAYS>  Minimum Inactivity Period in days for an `Identity` to be deleted [default: 90]
  -c, --concurrency <CONCURRENCY>      Number of concurrent requests when deleting the Identities [default: 10]
  -h, --help

```
