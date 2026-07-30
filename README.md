# genetec-ident-remover

Removes all inactive Identities which have not been modified within the last 90 days from a Genetec ClearID instance.

## Setup:

- Create an API integration and download the corresponding *.json file.
- Run the CLI with the following command in dry-run mode

```bash
genetec-ident-remover -k <keyfile_name>.json --dry-run

```

- Please carefully inspect the dumped identities and make sure no identities are included which should not be deleted.
  When ready:

```bash
genetec-ident-remover -k <keyfile_name>.json

```
