use std::{collections::HashMap, process::Command};

use crate::wpSetup::{Device, Devices};


pub(crate) static VIRT_HP : &'static str = "VirtOut";
pub(crate) static VIRT_MC : &'static str = "VirtIn";

///return vHp id for setting up
pub(super) fn setupVirtuals() -> Result<String,Box<dyn std::error::Error>>{
    //setup headphones + mic virtual ++ grab ids ? or setup then grab all?
    


    //make virt headphones
    if String::from_utf8( Command::new("sh").arg("-c")
        .arg(format!(
            "pw-cli list-objects | grep 'node.name = \"{VIRT_HP}\"' --count"
        ))
        .output()?.stdout )?.trim()
        == "0" //doesnt exist
    {
    //pw-cli create-node adapter factory.name=support.null-audio-sink node.name="VirtOut" node.description="AppsIn_HeadphonesOut" media.class=Audio/Sink object.linger=true
        Command::new("pw-cli")
        .args([
            // Call the node creation method
           "create-node", 
            // Use the adapter-based node creation logic
           "adapter", 
            // Use the null-sink to discard audio data
           "factory.name=support.null-audio-sink", 
            // Unique internal identifier for the graph
           &format!("node.name=\"{VIRT_HP}\""), 
            // Human-readable name for UI/tools
           &format!("node.description=\"{VIRT_HP}\""), 
            // Set as destination to accept incoming streams
           "media.class=Audio/Sink", 
            // Keep node alive after the creating client disconnects
           "object.linger=true", 
        ]).spawn()?;
    }else{
         println!("virt headphones exist")
    }


    //make virt mic
    if String::from_utf8( Command::new("sh").arg("-c")
        .arg(format!(
            "pw-cli list-objects | grep 'node.name = \"{VIRT_MC}\"' --count"
        ))
        .output()?.stdout )?.trim()
        == "0" //doesnt exist
    {
    //pw-cli create-node adapter factory.name=support.null-audio-sink node.name="VirtIn" node.description="MicsIn_AppsOut" media.class=Audio/Source/Virtual object.linger=true 
        Command::new("pw-cli")
        .args([
           "create-node",
           "adapter",
    //Mic - has to be sink factory since it will receive from mic
           "factory.name=support.null-audio-sink",
           &format!("node.name=\"{VIRT_MC}\""), 
           &format!("node.description=\"{VIRT_MC}\""), 
           "media.class=Audio/Source/Virtual",
           "object.linger=true",
        ]).spawn()?;
    }else{
         println!("virt mic exist")
    }



    setDefault()
}



// apps auto connect to current default even if it changes
pub fn setDefault() -> Result<String,Box<dyn std::error::Error>> {
    //VirtIn | VirtOut
    let res = String::from_utf8(
        Command::new("sh").arg("-c")
        .args([
            format!(r#"wpctl status | grep --perl-regexp --ignore-case "\\d+\. .*({VIRT_MC}|{VIRT_HP}).*(?=\[vol)" "#),
        ]).output()?.stdout
    )?;

    #[cfg(debug_assertions)] //disables on release
    println!("===\nres : {res}\n===");

    //wipe any existing defaults for clean integration
    Command::new("wpctl").arg("clear-default").output()?;
    
    
    let mut i=0; let mut vHpId: String="".into();
    for line in res.split("\n"){
        
        if let Some(id) = regex::regex!(r"\d+").find(line)
        && let id = id.as_str()
        {
            i = i+1;
            if(line.contains(VIRT_HP)){ vHpId=id.into(); }
            Command::new("wpctl").args(["set-default",id])
            .spawn()?;
            //default volume to max (wont actually affect anything)
            Command::new("wpctl").args(["set-volume",id,".5"])
            .spawn()?;
        }
    }


    if i!=2 { Err(format!(
        "Missing virt device - {}",
        if res.contains(VIRT_MC) { "headphones" }
        else if res.contains(VIRT_HP) {"mic"}
        else{ "Both" }
        ).into())
    }
    else{
        Ok(
            vHpId
        )
    }
}

///execute independent to setupVirtual so that if new devices connect etc.. can-connect them instead of remaking virts
pub(super) fn setupConnections(devs: &HashMap<Devices,Vec<Device>>) -> Result<(),Box<dyn std::error::Error>>
{
    //pw listobjs.. grep via device thing.. connect them up.. then set default

    //set default first then link up
    
    //TODO match against pwcli listobjs node.name or just name:output directly?
    
    // output (gives sound) | input (to receive sound) || cant link to same type
    //pw-link --outputs
    // pw-link --inputs
    // already curated.. find matching 'name' grab first 2 as (FL/FR) aux or not.. do link, done.
    // VirtOut:monitor || VirtIn:input

    let output = String::from_utf8(
        Command::new("pw-link").arg("--input").output()?
        .stdout
    )?;
    let mut aud_receivers = output.split("\n").filter_map(|x|
        if x.is_empty(){ None }
        else{ x.to_string().into() }
    ).collect::<Vec<_>>();

    let output = String::from_utf8(
        Command::new("pw-link").arg("--output").output()?
        .stdout
    )?;
    let mut aud_senders = output.split("\n").filter_map(|x|
        if x.is_empty(){ None }
        else{ x.to_string().into() }
    ).collect::<Vec<_>>(); 

    let mut i=0; let mut X="".to_string();


    // first.. check if device friendly && perform i count / filter run
    aud_senders.retain(|ad|
        //neutralise unbelonging
        devs.get(&Devices::Source).is_some_and(|f| f.iter().any(|d|
            ad.split(':').next().is_some_and(|x| x==d.pwId )
        ) )
        &&
        //take 2 method
        ad.split(':').next().is_some_and(|t|{
            if t==X{ i = i+1 } else{i=0; X=t.into()}; i<=1
        })
    );

    /* panic!("Check me:\n {}",
        serde_json::to_string_pretty( devs )?
        // serde_json::to_string_pretty(&aud_receivers)?,
        // serde_json::to_string_pretty(&aud_senders)?,
    ); */

    //Fix up aud_receivers
    aud_receivers.retain(|ad|{
        // println!("\n===\n{ad}\n===");
        //neutralise unbelonging
        devs.get(&Devices::Sink).is_some_and(|f| f.iter().any(|d|{
            // println!("PreCheck: {ad} vs {}",d.pwId);
            ad.split(':').next().is_some_and(|x| x==d.pwId )
        }) )
        &&
        //take 2 method
        ad.split(':').next().is_some_and(|t|{
            if t==X{ i = i+1 } else{i=0; X=t.into()}; i<=1
        })
    });


    #[cfg(debug_assertions)]
    println!("sorted pw o:{}", serde_json::to_string_pretty(&aud_senders)?);
    

    //finally link up Virts to auds
    #[cfg(debug_assertions)]
    println!("headphones:{}",aud_receivers.len());
    aud_receivers.iter().for_each(|ad|{
        // println!("CM: {ad:?}");
        // AUX0/FL vs AUX1/FR
        // AUX - default/physical | virtual/created

        let r = Command::new("pw-link")
        .args([
            &format!("{VIRT_HP}:{}",
                if ad.contains("_AUX0") || ad.contains("_FL"){ "monitor_FL" }
                else if ad.contains("_AUX1") || ad.contains("_FR"){ "monitor_FR" }
                else{" \"; exit 1; \"  "}
            )
            ,
            ad ])
        .output();

        #[cfg(debug_assertions)]
        println!("{r:?}");
    });

    #[cfg(debug_assertions)]
    println!("mics:{}",aud_senders.len());
    aud_senders.iter().for_each(|ad|{
        // AUX0/FL vs AUX1/FR
        // AUX - default/physical | virtual/created

        let _ = Command::new("pw-link")
        .args([
            ad,
            &format!("{VIRT_MC}:{}",
                if ad.contains("_AUX0") || ad.contains("_FL"){ "input_FL" }
                else if ad.contains("_AUX1") || ad.contains("_FR"){ "input_FR" }
                else{" \"; exit 1; \"  "}
            )
            ])
        .output();
    });

    
    Ok(())
}
