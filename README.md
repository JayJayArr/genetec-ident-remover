# genetec-ident-remover

Removes all inactive Identities which have not been modified within the last 90 days from a Genetec ClearID instance.
This has been tested against the Genetec ClearID-API identity enpoints v4.

## Prerequisites:

Create an API integration and download the corresponding *.json file. This includes all necessary information for authentication and endpoints.

## Usage:

For help please take a look at:

```bash
genetec-ident-remover --help
Usage: genetec-ident-remover [OPTIONS] -k <KEYFILE>

Options:
  -k <KEYFILE>                         Integration key-file from Genetec to authenticate
      --delete                         Deletes the found users
  -i, --inactive-days <INACTIVE_DAYS>  Minimum Inactivity Period in days for an `Identity` to be deleted [default: 90]
  -c, --concurrency <CONCURRENCY>      Number of concurrent requests when deleting the Identities [default: 10]
  -h, --help                           Print help
  -V, --version                        Print version
```

## Deleting

Run the CLI with the following command to have a look at the identities planned for deletion

```bash
genetec-ident-remover -k <keyfile_name>.json

```

- Please carefully inspect the dumped identities and make sure no identities are included which should not be deleted.
  When ready:

```bash
genetec-ident-remover -k <keyfile_name>.json --delete

```
