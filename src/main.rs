#![allow(non_snake_case)]

use std::env;
use std::process;
use std::path::PathBuf;
use std::io::{BufReader, Write};
use std::str;
use std::fs::File;
use quick_xml::events::Event;
use quick_xml::Reader;

//-----------------------------------------------------------------------------
// Macros
//-----------------------------------------------------------------------------
macro_rules! write_tag_i {
	($file:tt, $value:tt, $tag:tt, $forcewrite:tt) => {{
		
		let mut empty = false;
		if $value == 0 {empty = true;}

		if empty == false || $forcewrite == true
		{
			match write!($file, "\t\t<{}>{}</{}>\n", $tag, $value, $tag)
			{
				Ok(_) => {}
				Err(e) => {panic!("Error writing value {} for xml tag {}\n {:?}", $value, $tag, e)}
			}
		}
	}}
}
macro_rules! write_tag_s {
	($file:tt, $value:tt, $tag:tt, $forcewrite:tt) => {{
		
		let mut empty = false;
		if $value == "" {empty = true;}

		if empty == false || $forcewrite == true
		{
			let s: String;
			if $value.contains("&")
			{ s = $value.replace("&", "&amp;"); }
			else { s = $value.clone(); }

			match write!($file, "\t\t<{}>{}</{}>\n", $tag, s, $tag)
			{
				Ok(_) => {}
				Err(e) => {panic!("Error writing value {} for xml tag {}\n {:?}", $value, $tag, e)}
			}
		}
	}}
}



fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1)
    });

    let xmlpath = PathBuf::from(config.xmlfilepath);
    if xmlpath.exists() == false {
        println!("xml file not found at: {}", xmlpath.to_string_lossy());
        process::exit(4);
    }

    let data = MercOpinions::loadMercOpinions(&xmlpath);
    let mut pathOout = xmlpath.clone();
    pathOout.pop();
    pathOout.push("MercOpinions out.xml");
    data.saveMercOpinions(&pathOout);
}


struct Config {
    xmlfilepath: String,
}
impl Config {
    fn new(args: &[String]) -> Result<Config, String> {
        if args.len() < 2 {
            let errString = String::from("Not enough arguments!\nProvide path to JA2 1.13 xml file to be converted");
            return Err(errString);
        }

        let xmlfilepath = args[1].clone();

        Ok(Config {xmlfilepath})
    }
}


struct MercOpinions
{
    index: Vec<u8>,
    nicknames: Vec<String>,
    opinions: Vec<Vec<i32>>
}
impl MercOpinions 
{
    fn new() -> MercOpinions
    {
        let index = Vec::new();
        let nicknames = Vec::new();
        let opinions = Vec::new();

        return MercOpinions{index, nicknames, opinions};
    }

    fn loadMercOpinions(filepath: &PathBuf) -> MercOpinions
    {
        let mut mercOpinions = MercOpinions::new();

        let reader = Reader::from_file(filepath);
        match reader
        {
            Ok(mut reader) =>
            {
                reader.trim_text(true);
                let mut buf = Vec::new();
                loop 
                {
                    match reader.read_event_into(&mut buf) 
                    {
                        Err(element) => panic!("Error at position {}: {:?}", reader.buffer_position(), element),
                        Ok(Event::Eof) => break,

                        Ok(Event::Start(ref element)) => 
                        {
                            match element.name().as_ref() 
                            {			
                                b"OPINION" =>
                                {
                                    mercOpinions.readItem(&mut reader, &mut buf);
                                }
                                _ => {}
                            }
                        }
                        _ => ()
                    }
                    buf.clear();
                }
            }
            Err(e) =>
            {
                println!("Error {}", e);
                println!("Could not open file {}", filepath.display());
            }
        }
        return mercOpinions;
    }


    fn saveMercOpinions(&self, filepath: &PathBuf)
    {
        let mut buffer = Vec::new();
        // Write xml header before the xml data
        // write!(buffer, "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n").unwrap();

		write!(buffer, "<MERCOPINIONS>\n").unwrap();

        for i in &self.index
        {
	    	write!(buffer, "\t<OPINION>\n").unwrap();

            let value = i.clone();
            write_tag_i!(buffer, value, "uiIndex", true);
            
            let value = &self.nicknames[*i as usize];
            write_tag_s!(buffer, value, "zNickname", true);
        
            for j in 0..self.opinions[*i as usize].len()
            {
                let value = self.opinions[*i as usize][j];

                let mut empty = false;
                if value == 0 {empty = true;}
        
                if empty == false
                {
                    match write!(buffer, "\t\t<AnOpinion id = \"{}\" modifier = \"{}\"/>\n", j, value)
                    {
                        Ok(_) => {}
                        Err(e) => {panic!("Error writing value {} for xml tag {}\n {:?}", value, "AnOpinion", e)}
                    }
                }
        
            }

            write!(buffer, "\t</OPINION>\n").unwrap();
        }


		write!(buffer, "</MERCOPINIONS>\n").unwrap();

        println!("{}", &filepath.to_str().unwrap());
        std::fs::create_dir_all(filepath.parent().unwrap());
        let mut file = File::create(filepath).unwrap();
        file.write_all(&buffer);
    }

    pub fn readItem(&mut self, reader: &mut Reader<BufReader<std::fs::File>>, buf: &mut Vec<u8>)
	{
		loop 
		{
			match reader.read_event_into(buf) 
			{
				    Ok(Event::Start(e)) => 
				    {
					        let name = str::from_utf8(e.name().as_ref()).unwrap().to_string();
					        match e.name().as_ref()
					        {
			            		b"uiIndex" => { self.index.push(parseu8(reader, buf, &name)); }
						        b"zNickname" => { self.nicknames.push(parseString(reader, buf, b"szWeaponName")); }
			            		b"Opinion0" => { self.opinions.push(vec![parsei32(reader, buf, &name)]); }
			            		b"Opinion1" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion2" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion3" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion4" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion5" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion6" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion7" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion8" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion9" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion10" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion11" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion12" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion13" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion14" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion15" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion16" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion17" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion18" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion19" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion20" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion21" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion22" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion23" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion24" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion25" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion26" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion27" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion28" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion29" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion30" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion31" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion32" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion33" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion34" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion35" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion36" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion37" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion38" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion39" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion40" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion41" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion42" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion43" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion44" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion45" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion46" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion47" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion48" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion49" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion50" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion51" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion52" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion53" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion54" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion55" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion56" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion57" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion58" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion59" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion60" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion61" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion62" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion63" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion64" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion65" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion66" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion67" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion68" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion69" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion70" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion71" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion72" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion73" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion74" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion75" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion76" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion77" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion78" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion79" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion80" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion81" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion82" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion83" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion84" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion85" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion86" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion87" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion88" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion89" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion90" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion91" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion92" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion93" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion94" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion95" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion96" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion97" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion98" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion99" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion100" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion101" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion102" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion103" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion104" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion105" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion106" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion107" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion108" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion109" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion110" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion111" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion112" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion113" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion114" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion115" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion116" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion117" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion118" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion119" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion120" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion121" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion122" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion123" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion124" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion125" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion126" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion127" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion128" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion129" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion130" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion131" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion132" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion133" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion134" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion135" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion136" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion137" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion138" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion139" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion140" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion141" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion142" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion143" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion144" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion145" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion146" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion147" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion148" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion149" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion150" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion151" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion152" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion153" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion154" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion155" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion156" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion157" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion158" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion159" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion160" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion161" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion162" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion163" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion164" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion165" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion166" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion167" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion168" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion169" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion170" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion171" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion172" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion173" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion174" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion175" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion176" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion177" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion178" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion179" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion180" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion181" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion182" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion183" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion184" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion185" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion186" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion187" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion188" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion189" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion190" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion191" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion192" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion193" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion194" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion195" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion196" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion197" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion198" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion199" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion200" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion201" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion202" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion203" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion204" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion205" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion206" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion207" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion208" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion209" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion210" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion211" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion212" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion213" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion214" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion215" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion216" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion217" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion218" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion219" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion220" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion221" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion222" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion223" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion224" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion225" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion226" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion227" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion228" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion229" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion230" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion231" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion232" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion233" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion234" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion235" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion236" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion237" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion238" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion239" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion240" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion241" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion242" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion243" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion244" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion245" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion246" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion247" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion248" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion249" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion250" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion251" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion252" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion253" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
			            		b"Opinion254" => { self.opinions.last_mut().unwrap().push(parsei32(reader, buf, &name)); }
								_ => {}
						        }
				    }

				    Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
				    Ok(Event::End(ref element)) => 
				    {
					        match element.name().as_ref()
					        {
						            b"OPINION" => break,
						            _ => ()
					        }
				    }
				    _ => (),
			}
			buf.clear();
		}	
	}
}


//-----------------------------------------------------------------------------
// Functions
//-----------------------------------------------------------------------------
fn parseString(reader: &mut Reader<BufReader<File>>, buf: &mut Vec<u8>, tag: &[u8]) -> String
{
	loop {
		match reader.read_event_into(buf) 
		{
			Ok(Event::Text(e)) => {
				let value = e.unescape().unwrap().into_owned();
				return value;
			}
			Ok(Event::End(ref element)) => 
			{
					match element.name().as_ref()
					{
							tag => break,
							_ => ()
					}
			}
			Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
			_ => {}
		}
	}

	return "".to_string();
}

fn parsebool(reader: &mut Reader<BufReader<File>>, buf: &mut Vec<u8>, name: &str) -> bool
{
	loop {
		match reader.read_event_into(buf) 
		{
			Ok(Event::Text(e)) => {
				let value = e.unescape().unwrap().into_owned().parse::<u32>();
				match value
				{
					Ok(value) => {return value != 0;}
					_ => {println!("Error parsing value for tag {}", name); return false;}
				}
			}
			Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
			_ => {}
		}
	}
}

macro_rules! parsers {
	($($name:ident, $type:ty),*) => {
		
		$(fn $name(reader: &mut Reader<BufReader<File>>, buf: &mut Vec<u8>, name: &str) -> $type
		{
			loop {
				match reader.read_event_into(buf) 
				{
					Ok(Event::Text(e)) => {
						let value = e.unescape().unwrap().into_owned().parse::<$type>();
						match value
						{
							Ok(value) => {return value;}
							_ => {println!("Error parsing value for tag {} at position {}", name, reader.buffer_position()); return Default::default();}
						}
					}
					Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
					_ => {}
				}
			}
		})*
	};
}
parsers!(parseu8, u8, parsei8, i8, parseu16, u16, parsei16, i16, parseu32, u32, parsei32, i32, parseu64, u64, parsei64, i64, parsef32, f32);

