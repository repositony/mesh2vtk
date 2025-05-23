// crate modules
pub mod cli;
pub mod wrappers;

// crate includes
use cli::Cli;
use wrappers::CliVtkFormat;

// standard
use std::path::PathBuf;

// neutronics toolbox
use ntools::mesh::vtk::MeshToVtk;
use ntools::mesh::{Geometry, Group, Mesh};
use ntools::utils::f;

// external
use anyhow::Result;
use log::{debug, info, trace, warn};

/// Sets up logging at runtime to allow for multiple verbosity levels
pub fn init_logging(cli: &Cli) -> Result<()> {
    Ok(stderrlog::new()
        .module(module_path!())
        .quiet(cli.quiet)
        .verbosity(cli.verbose as usize + 2)
        .show_level(cli.verbose > 0)
        .color(stderrlog::ColorChoice::Auto)
        .timestamp(stderrlog::Timestamp::Off)
        .init()?)
}

/// Attempts to read a single targeted mesh from the file
pub fn try_meshtal_read(cli: &Cli) -> Result<Mesh> {
    if cli.quiet {
        Ok(ntools::mesh::read_target(&cli.file, cli.number)?)
    } else {
        Ok(ntools::mesh::read_target_pb(&cli.file, cli.number)?)
    }
}

/// Sanitise the output given and append the mesh tally id
pub fn output_path(mesh: &Mesh, cli: &Cli) -> PathBuf {
    let mut path = PathBuf::from(&cli.output);

    // take the file name provided
    let name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("fmesh");
    trace!("found the name \"{name}\"");

    // set the correct extension
    let extension = match cli.format {
        CliVtkFormat::Xml => match mesh.geometry {
            Geometry::Rectangular => "vtr",
            Geometry::Cylindrical => "vtu",
        },
        _ => "vtk",
    };
    trace!("set extension to \"{extension}\"");

    // append the mesh tally number to the name
    path.set_file_name(f!("{}_{}", name, mesh.id));
    path.set_extension(extension);
    debug!("File {:?}", path.file_name().unwrap());
    path
}

pub fn init_converter(mesh: &Mesh, cli: &Cli) -> MeshToVtk {
    let (energies, times) = get_targets(mesh, cli);
    trace!("energy idx {:?}", energies);
    trace!("time idx {:?}", times);

    if let Some(r) = cli.resolution {
        info!("Resolution set to {r}")
    };

    if cli.no_error {
        info!("Excluding error mesh from VTK")
    }

    MeshToVtk::builder()
        .include_errors(!cli.no_error)
        .byte_order(cli.endian.into())
        .compressor(cli.compressor.into())
        .resolution(cli.resolution.unwrap_or(1))
        .energy_groups(energies)
        .time_groups(times)
        .build()
}

fn get_targets(mesh: &Mesh, cli: &Cli) -> (Vec<usize>, Vec<usize>) {
    // simple case of --total flag seen
    if cli.total {
        debug!("Set targeted groups to 'Total' only");
        return (vec![mesh.n_ebins() - 1], vec![mesh.n_tbins() - 1]);
    }

    // then we want to find targeted groups by either index or absolute value
    match cli.absolute {
        false => parse_as_index(mesh, cli),
        true => parse_as_absolute(mesh, cli),
    }
}

fn parse_as_index(mesh: &Mesh, cli: &Cli) -> (Vec<usize>, Vec<usize>) {
    debug!("Parsing energy/time groups as indicies");
    (
        index_set(&cli.energy, mesh.n_ebins() - 1),
        index_set(&cli.time, mesh.n_tbins() - 1),
    )
}

fn index_set(targets: &[String], total_idx: usize) -> Vec<usize> {
    if targets.is_empty() {
        return (0..total_idx + 1).collect();
    }

    let mut indicies = targets_to_usize(targets);
    if targets.iter().any(|t| t.to_lowercase() == "total") {
        indicies.push(total_idx)
    };
    if indicies.is_empty() {
        debug!("Warning: Unable to parse indicies provided");
        warn!("  - {targets:?}");
        warn!("  - Falling back to all groups");
        indicies = (0..total_idx + 1).collect();
    } else {
        indicies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        indicies.dedup();
    };

    indicies
}

fn targets_to_usize(targets: &[String]) -> Vec<usize> {
    targets
        .iter()
        .filter_map(|group| group.parse::<usize>().ok())
        .collect()
}

fn parse_as_absolute(mesh: &Mesh, cli: &Cli) -> (Vec<usize>, Vec<usize>) {
    debug!("Parsing energy/time groups as absolute values");
    let energies = if cli.energy.is_empty() {
        (0..mesh.n_ebins()).collect()
    } else {
        energy_groups_to_index_set(mesh, &group_set(&cli.energy))
    };

    let times = if cli.time.is_empty() {
        (0..mesh.n_tbins()).collect()
    } else {
        time_groups_to_index_set(mesh, &group_set(&cli.time))
    };

    (times, energies)
}

fn group_set(targets: &[String]) -> Vec<Group> {
    let mut groups = targets_to_group(targets);
    if targets.iter().any(|t| t.to_lowercase() == "total") {
        groups.push(Group::Total)
    };
    groups.sort_by(|a, b| a.partial_cmp(b).unwrap());
    groups.dedup();
    groups
}

fn targets_to_group(targets: &[String]) -> Vec<Group> {
    targets
        .iter()
        .filter_map(|group| group.parse::<f64>().ok())
        .map(Group::Value)
        .collect::<Vec<Group>>()
}

fn energy_groups_to_index_set(mesh: &Mesh, groups: &[Group]) -> Vec<usize> {
    let indicies = groups
        .iter()
        .filter_map(|group| mesh.energy_index_from_group(*group).ok())
        .collect::<Vec<usize>>();

    // should never happen but you never know
    if indicies.is_empty() {
        warn!("Warning: No valid energy groups");
        warn!("  - Falling back to all groups");
        (0..mesh.n_ebins()).collect()
    } else {
        indicies
    }
}

fn time_groups_to_index_set(mesh: &Mesh, groups: &[Group]) -> Vec<usize> {
    let indicies = groups
        .iter()
        .filter_map(|group| mesh.time_index_from_group(*group).ok())
        .collect::<Vec<usize>>();

    // should never happen but you never know
    if indicies.is_empty() {
        warn!("Warning: No valid time groups");
        warn!("  - Falling back to all groups");
        (0..mesh.n_tbins()).collect()
    } else {
        indicies
    }
}
