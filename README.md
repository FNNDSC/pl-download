# Downloader _ChRIS_ Plugin

`pl-downloader` is a _ChRIS_ plugin which downloads files from URLs.

## Usage

`pl-downloader` has two modes: "single" mode (as a _fs_-type plugin),
or "bulk" mode (as a _ds_-type plugin).

### "Single" Mode

In "single" mode, `pl-downloader` downloads a file from one URL
and writes it to the output directory.

```shell
apptainer exec docker://ghcr.io/fnndsc/pl-downloader:latest downloader --url https://upload.wikimedia.org/wikipedia/commons/d/d1/Rubin_Observatory_and_Its_Target.jpg outputdir/
```

### "Bulk" Mode

In "bulk" mode, `pl-downloader` scans an input directory of files
where each file contains a list of URLs separated by whitespace.

```shell
cat > inputdir/urls.txt << EOF
https://upload.wikimedia.org/wikipedia/commons/a/ad/Devon_Rex_Cassini.jpeg
https://upload.wikimedia.org/wikipedia/commons/3/35/Saucer-eyed_Devon_Rex.jpg
https://upload.wikimedia.org/wikipedia/commons/1/10/Devon_Rex_Tortoiseshell.jpg
EOF
apptainer exec docker://ghcr.io/fnndsc/pl-downloader:latest downloader inputdir/ outputdir/
```
