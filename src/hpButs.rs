use std::process::Command;

use crate::setupDefaults::VIRT_HP;




pub(super) fn switchDevice(dir:bool) -> Result<(),Box<dyn std::error::Error>> {

    let outputs = String::from_utf8(
        Command::new("wpctl")
        .args(["list", "audio" ,"sinks"]).output()?.stdout
    )?
    .split("\n").filter_map(|x| if !x.is_empty() {x.to_string().into()} else {None} ).collect::<Vec<_>>()
    ;

    //grab id's index starts on * then +- done set new

    let mut def = outputs.iter().position(|x|{
        // println!("x : {x}");
        x.contains("*")
    }).ok_or("no default")? as i32;

    def = def + if dir { 1 } else {-1};

    let newDef = outputs.get(
        //can use modulo to wrap around?
        if def == outputs.len() as i32 { 0 }
        else if def<0{ outputs.len() - 1 }
        else{ def as usize }
    ).ok_or("failed get new default")?;

    Command::new("wpctl")
        .arg("set-default")
        .arg(
            newDef.chars().take_while(|c| c.is_digit(10))
            .collect::<String>()
        )
    .output();

    println!(
        "Default: {}"
        ,
        regex::regex!(r"\s+").split(newDef).collect::<Vec<_>>()
        .get(1).ok_or("devChangePrint")?
    );

    Ok(())
}

pub(super) fn volChange(inc:bool, step:i32) -> Result<(),Box<dyn std::error::Error>> {

    //changes if default device is virt - has to do ALL
    let outputs = String::from_utf8(
        Command::new("wpctl")
        .args(["list", "audio" ,"sinks"]).output()?.stdout
    )?
    .split("\n").filter_map(|x| if !x.is_empty() {x.to_string().into()} else {None} ).collect::<Vec<_>>()
    ;

    /// set `f32` to -1 for grab else set
    fn grabVol(id:&str,nV:f32) -> Result<f32,Box<dyn std::error::Error>>  {            //vec args have to outlive the the outter Command
        let NV = nV.clamp(0f32,1f32).to_string();
         
        #[cfg(debug_assertions)]
        println!("cmd: {:?}",
            Command::new("wpctl")
            .args(
            if nV < 0f32 {
                vec!["get-volume", id]
            }else{
                vec![
                    "set-volume",
                    id ,
                    &NV
                ]
            }
            )
        );
        
        let res = String::from_utf8(
        Command::new("wpctl")
            .args(
            if nV < 0f32 {
                vec!["get-volume", id]
            }else{
                vec![
                    "set-volume",
                    id ,
                    &NV
                ]
            }
            )
            .output()?.stdout
        )?;
        let res = res.split(":").collect::<Vec<_>>().get(1).ok_or("failGrabVol")?.to_string();
        
        #[cfg(debug_assertions)]
        println!("R: {res} -> {:?}",
            res.trim().parse::<f32>()
        );

        
        res.trim().parse()
            .map_err(|_| "grabVolErr".into() )
        
    }

    let def = outputs.iter().position(|x|{
        x.contains("*")
    }).map_or(vec!["".to_string();0], |x|
        regex::regex!(r"\s+").split(
        outputs.get(x).expect("volIndexErr")
        ).map(|x|x.to_string()).collect::<Vec<_>>()
    );
    let isVirt = def.get(1).is_some_and(|x|x.contains(VIRT_HP));


    //default device vol
    let cVol = grabVol(
        def.get(0).ok_or("indexFail".to_string())?
        , -1f32
    )?;

    let nVol = ((cVol * 100f32)
        + if inc {step} else {step*-1} as f32
    ) / 100f32; //0.xx => 0xx


    if !isVirt {
        
        grabVol(
            def.get(0).ok_or("indexErr")?
            , nVol
        )?;
        
    } else {
        outputs.iter().for_each(|dev|{
            let id = dev.split_once('\t').unwrap_or_default().0;

            grabVol(id, nVol);
        });
    }
    
    println!("{} {}%",
        def.get(1).ok_or("VlCprintErr")?
        ,
        (nVol*100f32).clamp(0f32,100f32) as i32
    );

    Ok(())
}
