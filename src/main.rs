use crate::{setupDefaults::VIRT_HP, wpSetup::Device};






mod wpSetup;
mod setupDefaults;
mod hpButs;



// coerce errors into box(generic) container that has dyn (trait) of Error
fn main() -> Result<(),Box<dyn std::error::Error>> {

    //arg - bool change dev .. true ++ false -- , x% vol step 3rd arg - volume change || none == setup/update
    // first arg is the cmd to execute it i.e. ./X or /a/b/X
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.len() {
        0 => {
    
        let myDevices = wpSetup::setup_devices()?;

        let vID = Device {
            pwId : VIRT_HP.to_string(),
            wpId : setupDefaults::setupVirtuals()?.parse::<i32>()?
        };

        setupDefaults::setupConnections(&myDevices)?;


        let h = myDevices.get(&wpSetup::Devices::Sink).ok_or("ERR")?;

        //default = wpctl list audio sinks | grep "*"
        },

        

        1 if let Some(b) = args.get(0)
            && let b = b.parse::<bool>().map_err(|_| format!("{b} : Bool parse fail") )?
         =>
        {
            hpButs::switchDevice(b)?;
        },

        

        2 if let Some(b) = args.get(0).and_then(|x| x.parse::<bool>().ok() )
        && let Some(i) = args.get(1).and_then(|x| x.parse::<i32>().ok() )
        =>
        {
           hpButs::volChange(b,i)?;
        }

        _ => {
            return Err("Match failed, program failed\n 0 args - setup | [bool] - change device | [bool, i32] - incr/decr vol by x%".into())
        }
    

    }
        
    Ok(())
}


