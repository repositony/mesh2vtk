// command line modules
use clap::builder::styling::{AnsiColor, Effects};
use clap::builder::Styles;
use clap::{arg, value_parser, Parser};

use crate::wrappers::{CliByteOrder, CliCompressor, CliVtkFormat};

/// Generalised conversion of meshtal files to visual toolkit formats
///
/// Examples
/// --------
///
///  Typical use:
///     $ mesh2vtk my_file.msht 104 -o my_output
///
///  Extract only the 'Total' energy and time groups:
///     $ mesh2vtk /path/to/file.msht 104 --total
///
///  Exclude voxel errors in the output:
///     $ mesh2vtk /path/to/file.msht 104 --no-error
///
///  Filter energy/time groups by index:
///     $ mesh2vtk /path/to/file.msht 104  \
///               --energy 0 2 6           \
///               --time 1 total
///
///  Filter energy/time groups by value:
///     $ mesh2vtk /path/to/file.msht 104  \
///               --energy 1.0 20.0 1e2    \
///               --time 1e12 total        \
///               --absolute
///
///  Output VTK in the old ASCII file format:
///     $ mesh2vtk /path/to/file.msht 104  \
///               --format legacy-ascii       
///
///  Alter basic mesh properties:
///     $ mesh2vtk /path/to/file.msht 104  \
///               --scale 1.0                 
///
/// Notes
/// -----
///
/// CuV results are weighted by volume of each contributing cell, and
/// VoidRecord::Off will fill in missing void voxels with 0.0 flux.
///
/// Run-on numbers without whitespace and broken exponential formatting
/// are handled.
///   - e.g. "1.00E+00-2.00E+00" => 1.00E+00 -2.00E+00
///   - e.g. "1.00+002" => 1.00E+002
///
/// Meshes with a single bin for EMESH/TMESH are labelled the 'Total'
/// group for consistency.
#[derive(Parser)]
#[command(
    verbatim_doc_comment,
    arg_required_else_help(true),
    after_help("Note: --help shows more information and examples"),
    term_width(76),
    hide_possible_values(true),
    override_usage("mesh2vtk <file> <id> [options]"),
    styles=custom_style()
)]
pub struct Cli {
    // * Positional
    /// Path to input meshtal file
    #[arg(name = "file")]
    pub file: String,

    /// Mesh tally identifier
    ///
    /// e.g. 104 for FMESH104:n
    #[arg(name = "number")]
    pub number: u32,

    // * Optional
    /// Only extract 'Total' energy/time groups
    ///
    /// By default all energy groups are included in the vtk. This equivalent to
    /// passing '--energy total --time total' as arguments.
    #[arg(help_heading("Mesh options"))]
    #[arg(long)]
    pub total: bool,

    /// Exclude error mesh from output files
    ///
    /// Error meshes are converted by default. The --no-error flag diables this
    /// behaviour to reduce the file size.
    #[arg(help_heading("Mesh options"))]
    #[arg(long)]
    pub no_error: bool,

    /// Multiply all results by a constant
    ///
    /// All results in the mesh are rescaled by the value provided. e.g. --scale
    /// 10.0 will multiply the result of every voxel by 10. Errors are relative
    /// and are therefore unchanged.
    #[arg(help_heading("Mesh options"))]
    #[arg(short, long)]
    #[arg(value_name = "num")]
    pub scale: Option<f64>,

    /// Filter energy group(s)
    ///
    /// By default all energy groups are included in the vtk. Specific energy
    /// groups can be specified by index. Values may be any combination of
    /// positive integers and the word 'total'.
    ///
    /// For filtering by real energy values in MeV rather than group index, use
    /// the --absolute falg.
    #[arg(help_heading("Mesh options"))]
    #[arg(short, long)]
    #[arg(value_parser, num_args = 1.., value_delimiter = ' ')]
    #[clap(required = false)]
    #[arg(conflicts_with = "total")]
    #[arg(value_name = "list")]
    pub energy: Vec<String>,

    /// Filter time group(s)
    ///
    /// By default all time groups are included in the vtk. Specific time
    /// groups can be specified by index. Values may be any combination of
    /// positive integers and the word 'total'.
    ///
    /// For filtering by real time values in shakes rather than group index, use
    /// the --absolute flag.
    #[arg(help_heading("Mesh options"))]
    #[arg(short, long)]
    #[arg(value_parser, num_args = 1.., value_delimiter = ' ')]
    #[clap(required = false)]
    #[arg(conflicts_with = "total")]
    #[arg(value_name = "list")]
    #[arg(allow_negative_numbers(true))]
    pub time: Vec<String>,

    /// Interpret filter values as MeV/shakes
    ///
    /// By default the values passed to the --energy and --time arguments are
    /// assumed to be group index. This is to avoid any issues resulting from
    /// poor output precisions in the mesh tally files.
    ///
    /// Using real values instead of the index is therefore opt-in, and use of
    /// this flag indicates preference. Warnings are given whenever using values
    /// over the index is a bad idea.
    #[arg(help_heading("Mesh options"))]
    #[arg(short, long)]
    pub absolute: bool,

    /// Name of output file (excl. extension)
    ///
    /// Defaults to `fmesh_<NUMBER>`, and will automatically append the mesh id
    /// and correct extension.
    #[arg(help_heading("Vtk options"))]
    #[arg(short, long)]
    #[arg(value_name = "name")]
    #[arg(hide_default_value(true))]
    #[arg(default_value = "fmesh")]
    pub output: String,

    /// VTK output format
    ///
    /// Available visual toolkit file formats:
    ///     > xml (default)
    ///     > legacy-ascii
    ///     > legacy-binary
    #[arg(help_heading("Vtk options"))]
    #[arg(short, long, value_enum)]
    #[arg(hide_default_value(true))]
    #[arg(default_value_t = CliVtkFormat::Xml)]
    #[arg(verbatim_doc_comment)]
    #[arg(value_name = "fmt")]
    pub format: CliVtkFormat,

    /// Cylindrical mesh resolution
    ///
    /// !! WARNING !!: Every vertex is defined explicitly, so large values will
    /// significantly increase memory usage and file size.
    ///
    /// Integer value for increasing angular resolution of cylindrical meshes.
    /// Cylinders are approximated to straight edge segments so it can be useful
    /// to round this off by splitting voxels into multiple smaller segments.
    ///
    /// e.g. 4 theta bins gives 4 edges and therefore looks square. Using
    /// `--resolution 3` generates 12 edges instead and looks more rounded.
    #[arg(help_heading("Vtk options"))]
    #[arg(long)]
    #[arg(value_name = "res")]
    #[arg(value_parser = value_parser!(u8).range(1..))]
    pub resolution: Option<u8>,

    /// Byte ordering (endian)
    ///
    /// Visit only reads big endian, most sytems are little endian.
    /// Defaults to big endian for convenience over performance.
    ///     > big-endian (default)
    ///     > little-endian
    #[arg(help_heading("Vtk options"))]
    #[arg(long, value_enum)]
    #[arg(hide_default_value(true))]
    #[arg(default_value_t = CliByteOrder::BigEndian)]
    #[arg(verbatim_doc_comment)]
    #[arg(value_name = "end")]
    pub endian: CliByteOrder,

    /// Comression method for xml
    ///
    /// Generally just use LZMA but other options are available.
    ///     > lzma (default)
    ///     > lz4
    ///     > zlib
    ///     > none
    #[arg(long, value_enum)]
    #[arg(help_heading("Vtk options"))]
    #[arg(hide_default_value(true))]
    #[arg(default_value_t = CliCompressor::LZMA)]
    #[arg(verbatim_doc_comment)]
    #[arg(value_name = "cmp")]
    pub compressor: CliCompressor,

    // * Flags
    /// Verbose logging (-v, -vv)
    ///
    /// If specified, the default log level of INFO is increased to DEBUG (-v)
    /// or TRACE (-vv). Errors and Warnings are always logged unless in quiet
    /// (-q) mode.
    #[arg(short, long)]
    #[arg(action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Supress all log output (overrules --verbose)
    #[arg(short, long)]
    pub quiet: bool,
}

/// Customise the colour styles for clap v4
fn custom_style() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Cyan.on_default() | Effects::BOLD | Effects::UNDERLINE)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Magenta.on_default())
}
