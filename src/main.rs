//! Command line tool to inspect and convert posvol files
#![doc(hidden)]

// standard library
use mesh2vtk::cli::Cli;
use ntools::mesh;

// external crates
use anyhow::Result;
use clap::Parser;
use log::{debug, info};

fn main() -> Result<()> {
    // set up the command line interface and logging
    let cli = Cli::parse();
    mesh2vtk::init_logging(&cli)?;

    // Get the mesh tallies
    info!("Reading {}", &cli.file);
    let mut mesh = mesh2vtk::try_meshtal_read(&cli)?;
    debug!("Mesh summary\n{mesh}");

    // Scale if needed
    if let Some(scale) = cli.scale {
        info!("Scaling results by {:.5e}", scale);
        mesh.scale(scale);
    }

    // Generate the vtk and write to file
    debug!("Initialising converter");
    let convertor = mesh2vtk::init_converter(&mesh, &cli);

    info!("Converting mesh to VTK object");
    let vtk = convertor.convert(&mesh);

    info!("Writing VTK to file");
    let path = mesh2vtk::output_path(&mesh, &cli);
    mesh::write_vtk(vtk, path, cli.format.into())?;

    Ok(())
}
