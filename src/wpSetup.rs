use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::setupDefaults::{VIRT_HP, VIRT_MC};


/**
`devices::Source` - mic/input
`devices::Sink` -- headphones/output
`devices::Stream` -- apps/audio streaming
*/
#[derive(strum::EnumIter,strum::Display,PartialEq,Eq,Hash, Debug,serde::Serialize,serde::Deserialize)]
pub(crate) enum Devices{
    Source,
    Sink,
}

#[derive(serde::Serialize,serde::Deserialize,Clone,Debug,)]
pub(crate) struct Device{
    pub(crate) wpId: i32,
    pub(crate) pwId: String
}

pub(super) fn setup_devices() -> Result<HashMap<Devices,Vec<Device>>,Box<dyn std::error::Error>>{

    let mut outputs  : Vec<Device> = vec![];
    let mut inputs = outputs.clone();

    let sinkRes = String::from_utf8(
        std::process::Command::new("wpctl")
        .args([
            "list", "audio", "sinks"
        ]).output()?.stdout
    )?;
    
    let sourceRes = String::from_utf8(
        std::process::Command::new("wpctl")
        .args([
            "list", "audio", "sources"
        ]).output()?.stdout
    )?;


    fn cliOutput2vecDev(inp:String,arr:&mut Vec<Device>)
    -> Result<(),Box<dyn std::error::Error>>
    {
        for line in inp.split("\n"){
            //for if runs while virts exist + avoids weird empties from split
            if line.contains(VIRT_HP) || line.contains(VIRT_MC) || line.is_empty() {
                continue;
            }
            let deets = regex::regex!(r"\s+").split(line)
                .take(2)
                .collect::<Vec<_>>();

            arr.push(Device {
                 wpId: deets[0].parse::<i32>()?,
                 pwId: deets[1].into()
            });
        }
        Ok(())
    }

    cliOutput2vecDev(sourceRes,&mut inputs )?;
    cliOutput2vecDev(sinkRes,&mut outputs )?;

    Ok(
        HashMap::from([
            (Devices::Source , inputs),
            (Devices::Sink , outputs),
    ]) )
}


