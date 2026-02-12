# irma-viz

A rust crate to replace R plotting dependencies in IRMA. 

With a modular backend, editable configs, and plotting by plotters.

## Usage

### Required arguments:

`--alleles-tsv`
`--min-aq`
`--min-f`
`--min-tcc`
`--min-conf`

### Config

Plots can be toggled, and settings can be changed in `config.toml`. These defaults can also be overridden by command line arguments.

### Example

`./irma-viz --alleles-tsv alleles.tsv --min-aq 1.1 --min-f 1.2 --min-tcc 1.3 --min-conf 1.4`