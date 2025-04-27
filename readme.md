# MCNP Mesh to VTK conversion (`mesh2vtk`)

[![GitHub release](https://img.shields.io/github/v/release/repositony/mesh2vtk?include_prereleases)](https://github.com/repositony/mesh2vtk/releases/latest)

Command line tool to convert MCNP mesh tallies to Visual ToolKit (VTK) formats.

```text
Usage: mesh2vtk <file> <id> [options]

Arguments:
  <file>                  Path to input meshtal file
  <number>                Mesh tally identifier

Options:
  -v, --verbose...        Verbose logging (-v, -vv)
  -q, --quiet             Supress all log output (overrules --verbose)
  -h, --help              Print help (see more with '--help')

Mesh options:
  -t, --total             Only extract 'Total' energy/time groups
  -s, --scale <num>       Multiply all results by a constant
      --energy <list>...  Filter energy group(s)
      --time <list>...    Filter time group(s)
  -a, --absolute          Filter by MeV/shakes rather than index
      --no-error          Exclude error mesh from output files

Vtk options:
  -o, --output <name>     Name of output file (excl. extension)
  -f, --format <fmt>      VTK output format
      --resolution <res>  Cylindrical mesh resolution
      --endian <end>      Byte ordering (endian)
      --compressor <cmp>  Comression method for xml

Note: --help shows more information and examples
```

Help is printed with the `-h` flag, and `--help` will show default values,
examples, and any important behaviour.

## Overview

### Supported output formats

For more detail, see the `OUT` keyword for the `FMESH` card definition in
the [MCNPv6.2](https://mcnp.lanl.gov/pdf_files/TechReport_2017_LANL_LA-UR-17-29981_WernerArmstrongEtAl.pdf)
or [MCNPv6.3](https://mcnpx.lanl.gov/pdf_files/TechReport_2022_LANL_LA-UR-22-30006Rev.1_KuleszaAdamsEtAl.pdf)
user manuals.

| Output format | Supported? | Description                                         |
| ------------- | ---------- | --------------------------------------------------- |
| XDMF/HDF5     | >2.0.0     | MCNP >6.3 runtape HDF5 format                       |
| COL           | Yes        | Column data (MCNP default)                          |
| CF            | Yes        | Column data including voxel volume                  |
| IJ            | Yes        | 2D matrix of I (col) and J (row) data, grouped by K |
| IK            | Yes        | 2D matrix of I (col) and K (row) data, grouped by J |
| JK            | Yes        | 2D matrix of J (col) and K (row) data, grouped by I |
| CUV (UKAEA)   | Yes        | UKAEA Cell-under-Voxel column data                  |
| NONE          | N/A        | `NONE` or unknown output format                     |

### Supported mesh geometries

All functionality is fully supported for rectangular and cylindrical meshes.

| Mesh geometry | Supported? | MCNP designators |
| ------------- | ---------- | ---------------- |
| Rectangular   | Yes        | rec, xyz         |
| Cylindrical   | Yes        | cyl, rzt         |
| Spherical     | No         | sph, rpt         |

Spherical meshes are not supported because barely anyone knows about them, let
alone uses them. They are therefore a low priority, but raise an issue if anyone
needs it.

## Install

Download and unpack the latest binary executable release [here](https://github.com/repositony/mesh2vtk/releases/latest) for running in a terminal/powershell.

### Linux/MacOS

Unpack the relevant executable from the [latest release](https://github.com/repositony/mesh2vtk/releases/latest).

```bash
# Linux
tar -xJf mesh2vtk-x86_64-unknown-linux-gnu.tar.xz  # Generic linux
tar -xJf mesh2vtk-aarch64-unknown-linux-gnu.tar.xz # ARM64 Linux

# MacOS
tar -xJf mesh2vtk-x86_64-apple-darwin.tar.xz       # Intel macOS
tar -xJf mesh2vtk-aarch64-apple-darwin.tar.xz      # Apple Silicon macOS
```

And either run from there or add the executable to your `$PATH`.

```bash
./mesh2vtk -h
```

### Windows

Extract `mesh2vtk-x86_64-pc-windows-msvc.zip` from the [latest release](https://github.com/repositony/mesh2vtk/releases/latest).

Navigate to this folder and run from powershell.

```bash
.\mesh2vtk.exe -h
```

This may be set as an alias for convenience.

```powershell
Set-Alias -Name mesh2vtk -Value C:\Path\To\Folder\mesh2vtk.exe
```

## Examples

### Only use the 'Total' energy/time groups

Often only the `Total` energy/time bins are of interest, and a quick way of
only converting this subset is provided.

```bash
# Extract only the 'Total' energy and time groups
mesh2vtk /path/to/meshtal.msht 104 --total
```

### Reduce VTK file size

Excluding uncertainty meshes from output cuts file sizes in half.

This may be desirable for extremely large meshtal files.

```bash
# Extract every energy and time group, with corresponding error meshes
mesh2vtk /path/to/meshtal.msht 104 --no-error
```

### Choose specific energy/time groups

If specific energy or time groups are required they can be filtered **by group
index**.

```bash
# Extract specific energy and time groups by index
mesh2vtk /path/to/meshtal.msht 104  \
            --energy 0 2 6          \
            --time 1 total
```

The keyword `total` can be used for the 'Total' energy/time groups for
convenience.

Filtering by index avoids the precision issues that accompany the extremely
limited MCNPv6.2 output formatting. However, if you really want to use absolute
values in units of `MeV` and `shakes` then include the `--absolute` flag.

```bash
# Extract specific energy and time groups by MeV/shakes values
mesh2vtk /path/to/meshtal.msht 104  \
            --energy 1.0 20.0 1e2   \
            --time 1e12 total       \
            --absolute
```

For intuitive use, the groups correspond with values defined on the `EMESH`
and `TMESH` cards.

*Note: mesh tallies label groups by the upper bounds defined on EMESH/TMESH
cards. i.e the energy group `1.0` corresponds to the `0.0>=E<1.0` bin,
though in reality 1 MeV particle would end up in the next group up.*

### Rescale all results

MCNP normalises everything so it is often the case that the results must be
rescaled to provide physical values.

```bash
# Rescale the result by a constant multiplier, e.g 6.50E+18 neutrons/s
mesh2vtk /path/to/meshtal.msht 104 --scale 6.50E+18
```

Mesh rotations are a work in progress.

### Change the output file name

By default the file prefix is `fmesh`, so the output files will be
`fmesh_<number>.vtk`. This may be changed as needed.

```bash
# Change the output name to `myvtk_104.vtr`
mesh2vtk /path/to/meshtal.msht 104 --output myvtk
```

### Change the Vtk format

Most useful may be the ability to decide on output formats. XML and legacy
formats are supported, with both ascii and binary variants.

```bash
# Output as a binary vtk with legacy formatting
mesh2vtk /path/to/meshtal.msht 104 --format legacy-binary
```

### Specify XML compression and binary byte order

For more advanced usage, things like byte ordering and xml compression
methods are also configurable.

```bash
# Output as an xml using lzma and setting the byte order to big endian
mesh2vtk /path/to/meshtal.msht 104      \
        --format xml                    \
        --compresser lzma               \
        --endian big-endian
```

*Note - [VisIt](https://visit-dav.github.io/visit-website/index.html) only
reads big-endian, but most sytems are natively little-endian. For personal
convenience the default is big, but I am open to arguments for little endian
as the default.*

### Plotting cylindrical meshes

There is no VTK representation of cylindrical meshes, so an unstructured
mesh is generated from verticies based on the RZT bounds.

Unfortunately, this can result in "low-resolution" plots for meshes with
few theta bins. The number of theta bins can be increased to round off these
edges. This simply subdivides the voxels by an integer number of theta bins.

![Cylindrical mesh resolution option](https://github.com/repositony/meshtal/blob/main/data/assets/cylindrical_mesh_resolution.png)

```bash
# Subdivide voxels into 3 to round off cylindrical unstructured mesh
mesh2vtk /path/to/meshtal.msht 104 --resolution 3
```

Note that this will increase the file size and memory usage significantly
in some cases.
