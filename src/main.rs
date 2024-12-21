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
macro_rules! write_tag_f {
	($file:tt, $value:tt, $tag:tt, $forcewrite:tt) => {{
		
		let mut empty = false;
		if $value == 0.0 {empty = true;}

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

//-----------------------------------------------------------------------------
// Main
//-----------------------------------------------------------------------------
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1)
    });

    let xmlpath = PathBuf::from(config.xmlfilepath);
    if xmlpath.exists() == false 
	{
        println!("xml file not found at: {}", xmlpath.to_string_lossy());
        process::exit(4);
    }

	if xmlpath.ends_with("MercOpinions.xml") 
	{
		
		let data = MercOpinions::loadMercOpinions(&xmlpath);
		let mut pathOout = xmlpath.clone();
		pathOout.pop();
		pathOout.push("MercOpinions out.xml");
		data.saveMercOpinions(&pathOout);
	}
	else if xmlpath.ends_with("Items.xml") 
	{
		
		let data = Items::loadItems(&xmlpath);
		let mut pathOout = xmlpath.clone();
		pathOout.pop();
		pathOout.push("Items out.xml");
		data.saveItems(&pathOout);
	}
}


//-----------------------------------------------------------------------------
// Structs
//-----------------------------------------------------------------------------
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


struct InvType
{
	szItemDesc: String,
	szBRDesc: String,
	szItemName: String,
	szLongItemName: String,
	szBRName: String,
	defaultattachments: Vec<u16>,

	nasAttachmentClass: u64,
	nasLayoutClass: u64,
	ulAvailableAttachmentPoint: u64,
	ulAttachmentPoint: u64,
	usItemFlag: u64, // bitflags to store various item properties (better than introducing 64 BOOLEAN values). If I only had thought of this earlier....
	usItemFlag2: u64, // bitflags to store various item properties

	uiIndex: u32,
	usItemClass: u32,
	attachmentclass: u32, // attachmentclass used
	drugtype: u32, // this flagmask determines what different components are used in a drug, which results in different effects
	foodtype: u32,
	usActionItemFlag: u32, // Flugente: a flag that is necessary for transforming action items to objects with new abilities (for now, tripwire networks and directional explosives)
	clothestype: u32, // Flugente: clothes type that 'links' to an entry in Clothes.xml

	//zilpin: pellet spread patterns externalized in XML
	// spreadPattern: i32,
	spreadPattern: String,

	alcohol: f32,
	// HEADROCK HAM 4: New modifiers that do not require a stance array, since they affect the gun objectively, not
	// subjectively.
	RecoilModifierX: f32,
	RecoilModifierY: f32,
	scopemagfactor: f32,
	projectionfactor: f32,
	usOverheatingCooldownFactor: f32,			// every turn/5 seconds, a gun's temperature is lowered by this amount
	overheatTemperatureModificator: f32,			// percentage modifier of heat a shot generates (read from attachments)
	overheatCooldownModificator: f32,			// percentage modifier of cooldown amount (read from attachments, applies to guns & barrels)
	overheatJamThresholdModificator: f32,		// percentage modifier of a gun's jam threshold (read from attachments)
	overheatDamageThresholdModificator: f32,		// percentage modifier of a gun's damage threshold (read from attachments)
	dirtIncreaseFactor: f32, // Flugente: advanced repair/dirt system. One shot causes this much dirt on a gun
	fRobotDamageReductionModifier: f32, // rftr: robot attachments

	// STAND/CROUCH/PRONE_MODIFIERS
	flatbasemodifier: [i16; 3],
	percentbasemodifier: [i16; 3],
	flataimmodifier: [i16; 3],
	percentaimmodifier: [i16; 3],
	percentcapmodifier: [i16; 3],
	percenthandlingmodifier: [i16; 3],
	percentdropcompensationmodifier: [i16; 3],
	maxcounterforcemodifier: [i16; 3],
	counterforceaccuracymodifier: [i16; 3],
	targettrackingmodifier: [i16; 3],
	aimlevelsmodifier: [i16; 3],

	//Madd: Common Attachment Framework:  attach items based on matching connection points rather than using the old long attachmentinfo method
	ubClassIndex: u16,
	ubGraphicNum: u16,
	ubWeight: u16, //2 units per kilogram, roughly 1 unit per pound
	ItemSize: u16,
	usPrice: u16,
	discardedlauncheritem: u16,
	randomitem: u16, // Flugente: a link to RandomItemsClass.xml. Out of such an item, a random object is created, depending on the entries in the xml
	usBuddyItem: u16, // Flugente: item is connected to another item. Type of connection depends on item specifics
	usRiotShieldStrength: u16,	// Flugente: riot shields. strength of shield
	usRiotShieldGraphic: u16,	// Flugente: riot shields. graphic of shield (when deployed in tactical, taken from Tilecache/riotshield.sti)

	percentnoisereduction: i16,
	bipod: i16,
	tohitbonus: i16,
	bestlaserrange: i16,
	rangebonus: i16,
	percentrangebonus: i16,
	aimbonus: i16,
	minrangeforaimbonus: i16,
	percentapreduction: i16,
	percentstatusdrainreduction: i16,
	bloodieditem: i16,
	hearingrangebonus: i16,
	visionrangebonus: i16,
	nightvisionrangebonus: i16,
	dayvisionrangebonus: i16,
	cavevisionrangebonus: i16,
	brightlightvisionrangebonus: i16,
	itemsizebonus: i16,
	damagebonus: i16,
	meleedamagebonus: i16,
	magsizebonus: i16,
	percentautofireapreduction: i16,
	autofiretohitbonus: i16,
	APBonus: i16,
	rateoffirebonus: i16,
	burstsizebonus: i16,
	bursttohitbonus: i16,
	percentreadytimeapreduction: i16,
	bulletspeedbonus: i16,
	percentreloadtimeapreduction: i16,
	percentburstfireapreduction: i16,
	camobonus: i16,
	stealthbonus: i16,
	urbanCamobonus: i16,
	desertCamobonus: i16,
	snowCamobonus: i16,
	PercentRecoilModifier: i16,
	percentaccuracymodifier: i16,
	usSpotting: i16, // Flugente: spotting effectiveness
	sBackpackWeightModifier: i16, // JMich: BackpackClimb modifier to weight calculation to climb.
	sFireResistance: i16,

	ubAttachToPointAPCost: u8, // cost to attach to any matching point
	ubCursor: u8,
	ubGraphicType: u8,
	ubPerPocket: u8,
	ubCoolness: u8,
	percenttunnelvision: u8,
	ubAttachmentSystem: u8, //Item availability per attachment system: 0 = both, 1 = OAS, 2 = NAS
	CrowbarModifier: u8,
	DisarmModifier: u8,
	usHackingModifier: u8,
	usBurialModifier: u8, // Flugente: a modifier for burial effectiveness
	usDamageChance: u8, //  Flugente: advanced repair/dirt system. Chance that damage to the status will also damage the repair threshold
	usFlashLightRange: u8, // Flugente: range of a flashlight (an item with usFlashLightRange > 0 is deemed a flashlight)
	usItemChoiceTimeSetting: u8, // Flugente: determine wether the AI should pick this item for its choices only at certain times
	ubSleepModifier: u8, // silversurfer: item provides breath regeneration bonus while resting
	usPortionSize: u8,			// Flugente: for consumables: how much of this item is consumed at once
	usAdministrationModifier: u8, // Flugente: a modifier for administration effectiveness
	inseparable: u8, //Madd:Normally, an inseparable attachment can never be removed.  
	//But now we will make it so that these items can be replaced, but still not removed directly.
	//0 = removeable (as before)
	//1 = inseparable (as before)
	//2 = inseparable, but replaceable

	bSoundType: i8,
	bReliability: i8,
	bRepairEase: i8,
	LockPickModifier: i8,
	RepairModifier: i8,
	randomitemcoolnessmodificator: i8, // Flugente: a link to RandomItemsClass.xml. alters the allowed maximum coolness a random item can have
	bRobotStrBonus: i8, // rftr: robot attachments
	bRobotAgiBonus: i8, // rftr: robot attachments
	bRobotDexBonus: i8, // rftr: robot attachments
	bRobotTargetingSkillGrant: i8, // rftr: robot attachments
	bRobotChassisSkillGrant: i8, // rftr: robot attachments
	bRobotUtilitySkillGrant: i8, // rftr: robot attachments
	iTransportGroupMinProgress: i8, // rftr: the progress bounds that allow a transport group to drop an item
	iTransportGroupMaxProgress: i8, // rftr: the progress bounds that allow a transport group to drop an item

	// These are not part of INVTYPE in the source code, but they are also read from items.xml
	BR_NewInventory: u8,
	BR_UsedInventory: u8,
	BR_ROF: i16,
}
impl InvType{
	fn new() -> InvType
	{
		let inv = InvType 
		{
			szItemDesc: String::new(),
			szBRDesc : String::new(),
			szItemName : String::new(),
			szLongItemName : String::new(),
			szBRName : String::new(),
			defaultattachments : Vec::new(),
		
			nasAttachmentClass : 0,
			nasLayoutClass : 0,
			ulAvailableAttachmentPoint : 0,
			ulAttachmentPoint : 0,
			usItemFlag : 0, // bitflags to store various item properties (better than introducing 64 BOOLEAN values). If I only had thought of this earlier....
			usItemFlag2 : 0, // bitflags to store various item properties
		
			uiIndex : 0,
			usItemClass : 0,
			attachmentclass : 0, // attachmentclass used
			drugtype : 0, // this flagmask determines what different components are used in a drug, which results in different effects
			foodtype : 0,
			usActionItemFlag : 0, // Flugente : a flag that is necessary for transforming action items to objects with new abilities (for now, tripwire networks and directional explosives)
			clothestype : 0, // Flugente : clothes type that 'links' to an entry in Clothes.xml
		
			spreadPattern : String::new(),
		
			alcohol : 0.0f32,
			RecoilModifierX : 0.0f32,
			RecoilModifierY : 0.0f32,
			scopemagfactor : 0.0f32,
			projectionfactor : 0.0f32,
			usOverheatingCooldownFactor : 0.0f32,			// every turn/5 seconds, a gun's temperature is lowered by this amount
			overheatTemperatureModificator : 0.0f32,			// percentage modifier of heat a shot generates (read from attachments)
			overheatCooldownModificator : 0.0f32,			// percentage modifier of cooldown amount (read from attachments, applies to guns & barrels)
			overheatJamThresholdModificator : 0.0f32,		// percentage modifier of a gun's jam threshold (read from attachments)
			overheatDamageThresholdModificator : 0.0f32,		// percentage modifier of a gun's damage threshold (read from attachments)
			dirtIncreaseFactor : 0.0f32, // Flugente : advanced repair/dirt system. One shot causes this much dirt on a gun
			fRobotDamageReductionModifier : 0.0f32, // rftr : robot attachments
		
			flatbasemodifier : [0; 3],
			percentbasemodifier : [0; 3],
			flataimmodifier : [0; 3],
			percentaimmodifier : [0; 3],
			percentcapmodifier : [0; 3],
			percenthandlingmodifier : [0; 3],
			percentdropcompensationmodifier : [0; 3],
			maxcounterforcemodifier : [0; 3],
			counterforceaccuracymodifier : [0; 3],
			targettrackingmodifier : [0; 3],
			aimlevelsmodifier : [0; 3],
		
			//Madd : Common Attachment Framework :  attach items based on matching connection points rather than using the old long attachmentinfo method
			ubClassIndex : 0,
			ubGraphicNum : 0,
			ubWeight : 0, //2 units per kilogram, roughly 1 unit per pound
			ItemSize : 0,
			usPrice : 0,
			discardedlauncheritem : 0,
			randomitem : 0, // Flugente : a link to RandomItemsClass.xml. Out of such an item, a random object is created, depending on the entries in the xml
			usBuddyItem : 0, // Flugente : item is connected to another item. Type of connection depends on item specifics
			usRiotShieldStrength : 0,	// Flugente : riot shields. strength of shield
			usRiotShieldGraphic : 0,	// Flugente : riot shields. graphic of shield (when deployed in tactical, taken from Tilecache/riotshield.sti)
		
			percentnoisereduction : 0,
			bipod : 0,
			tohitbonus : 0,
			bestlaserrange : 0,
			rangebonus : 0,
			percentrangebonus : 0,
			aimbonus : 0,
			minrangeforaimbonus : 0,
			percentapreduction : 0,
			percentstatusdrainreduction : 0,
			bloodieditem : 0,
			hearingrangebonus : 0,
			visionrangebonus : 0,
			nightvisionrangebonus : 0,
			dayvisionrangebonus : 0,
			cavevisionrangebonus : 0,
			brightlightvisionrangebonus : 0,
			itemsizebonus : 0,
			damagebonus : 0,
			meleedamagebonus : 0,
			magsizebonus : 0,
			percentautofireapreduction : 0,
			autofiretohitbonus : 0,
			APBonus : 0,
			rateoffirebonus : 0,
			burstsizebonus : 0,
			bursttohitbonus : 0,
			percentreadytimeapreduction : 0,
			bulletspeedbonus : 0,
			percentreloadtimeapreduction : 0,
			percentburstfireapreduction : 0,
			camobonus : 0,
			stealthbonus : 0,
			urbanCamobonus : 0,
			desertCamobonus : 0,
			snowCamobonus : 0,
			PercentRecoilModifier : 0,
			percentaccuracymodifier : 0,
			usSpotting : 0, // Flugente : spotting effectiveness
			sBackpackWeightModifier : 0, // JMich : BackpackClimb modifier to weight calculation to climb.
			sFireResistance : 0,

			ubAttachToPointAPCost : 0, // cost to attach to any matching point
			ubCursor : 0,
			ubGraphicType : 0,
			ubPerPocket : 0,
			ubCoolness : 0,
			percenttunnelvision : 0,
			ubAttachmentSystem : 0, //Item availability per attachment system : 0 : both, 1 : OAS, 2 : NAS
			CrowbarModifier : 0,
			DisarmModifier : 0,
			usHackingModifier : 0,
			usBurialModifier : 0, // Flugente : a modifier for burial effectiveness
			usDamageChance : 0, //  Flugente : advanced repair/dirt system. Chance that damage to the status will also damage the repair threshold
			usFlashLightRange : 0, // Flugente : range of a flashlight (an item with usFlashLightRange > 0 is deemed a flashlight)
			usItemChoiceTimeSetting : 0, // Flugente : determine wether the AI should pick this item for its choices only at certain times
			ubSleepModifier : 0, // silversurfer : item provides breath regeneration bonus while resting
			usPortionSize : 0,			// Flugente : for consumables : how much of this item is consumed at once
			usAdministrationModifier : 0, // Flugente : a modifier for administration effectiveness
			inseparable : 0, //Madd :Normally, an inseparable attachment can never be removed.  
			
			bSoundType : 0,
			bReliability : 0,
			bRepairEase : 0,
			LockPickModifier : 0,
			RepairModifier : 0,
			randomitemcoolnessmodificator : 0, // Flugente : a link to RandomItemsClass.xml. alters the allowed maximum coolness a random item can have
			bRobotStrBonus : 0, // rftr : robot attachments
			bRobotAgiBonus : 0, // rftr : robot attachments
			bRobotDexBonus : 0, // rftr : robot attachments
			bRobotTargetingSkillGrant : 0, // rftr : robot attachments
			bRobotChassisSkillGrant : 0, // rftr : robot attachments
			bRobotUtilitySkillGrant : 0, // rftr : robot attachments
			iTransportGroupMinProgress : 0, // rftr : the progress bounds that allow a transport group to drop an item
			iTransportGroupMaxProgress : 0, // rftr: the progress bounds that allow a transport group to drop an item

			BR_NewInventory: 0,
			BR_UsedInventory: 0,
			BR_ROF: 0,
		};

		return inv;
	}
}
struct Items{
	items: Vec<InvType>
}
impl Items 
{
    fn new() -> Items
    {
        let items = Vec::new();

        return Items{items};
    }

    fn loadItems(filepath: &PathBuf) -> Items
    {
        let mut itemdata = Items::new();

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
                                b"ITEM" =>
                                {
									itemdata.items.push(InvType::new());
                                    itemdata.readItem(&mut reader, &mut buf);
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
        return itemdata;
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
					let item = self.items.last_mut().unwrap();
					match e.name().as_ref()
					{
						b"uiIndex" => { item.uiIndex = parseu32(reader, buf, &name); }
						b"szItemName" => { item.szItemName = parseString(reader, buf, b"szItemName"); }
						b"szLongItemName" => { item.szLongItemName = parseString(reader, buf, b"szLongItemName"); }
						b"szItemDesc" => { item.szItemDesc = parseString(reader, buf, b"szItemDesc"); }
						b"szBRName" => { item.szBRName = parseString(reader, buf, b"szBRName"); }
						b"szBRDesc" => { item.szBRDesc = parseString(reader, buf, b"szBRDesc"); }
						b"usItemClass" => { item.usItemClass = parseu32(reader, buf, &name); }
						b"AttachmentClass" => { item.attachmentclass = parseu32(reader, buf, &name); }
						b"nasAttachmentClass" => { item.nasAttachmentClass = parseu64(reader, buf, &name); }
						b"nasLayoutClass" => { item.nasLayoutClass = parseu64(reader, buf, &name); }
						b"AvailableAttachmentPoint" => { item.ulAvailableAttachmentPoint = parseu64(reader, buf, &name); }
						b"AttachmentPoint" => { item.ulAttachmentPoint = parseu64(reader, buf, &name); }
						b"AttachToPointAPCost" => { item.ubAttachToPointAPCost = parseu8(reader, buf, &name); }
						b"ubClassIndex" => { item.ubClassIndex = parseu16(reader, buf, &name); }
						b"ubCursor" => { item.ubCursor = parseu8(reader, buf, &name); }
						b"bSoundType" => { item.bSoundType = parsei8(reader, buf, &name); }
						b"ubGraphicType" => { item.ubGraphicType = parseu8(reader, buf, &name); }
						b"ubGraphicNum" => { item.ubGraphicNum = parseu16(reader, buf, &name); }
						b"ubWeight" => { item.ubWeight = parseu16(reader, buf, &name); }
						b"ubPerPocket" => { item.ubPerPocket = parseu8(reader, buf, &name); }
						b"ItemSize" => { item.ItemSize = parseu16(reader, buf, &name); }
						b"ItemSizeBonus" => { item.itemsizebonus = parsei16(reader, buf, &name); }
						b"usPrice" => { item.usPrice = parseu16(reader, buf, &name); }
						b"ubCoolness" => { item.ubCoolness = parseu8(reader, buf, &name); }
						b"bReliability" => { item.bReliability = parsei8(reader, buf, &name); }
						b"bRepairEase" => { item.bRepairEase = parsei8(reader, buf, &name); }
						b"DrugType" => {item.drugtype = parseu32(reader, buf, &name);}
						b"FoodType" => {item.foodtype = parseu32(reader, buf, &name);}
						b"usActionItemFlag" => {item.usActionItemFlag = parseu32(reader, buf, &name);}
						b"clothestype" => {item.clothestype = parseu32(reader, buf, &name);}

						b"spreadPattern" => {item.spreadPattern = parseString(reader, buf, b"spreadPattern"); }

						b"Alcohol" => {item.alcohol = parsef32(reader, buf, &name);}
						b"ScopeMagFactor" => {item.scopemagfactor = parsef32(reader, buf, &name);}
						b"ProjectionFactor" => {item.projectionfactor = parsef32(reader, buf, &name);}
						b"RecoilModifierX" => {item.RecoilModifierX = parsef32(reader, buf, &name);}
						b"RecoilModifierY" => {item.RecoilModifierY = parsef32(reader, buf, &name);}
						b"PercentRecoilModifier" => {item.PercentRecoilModifier = parsei16(reader, buf, &name);}
						b"PercentAccuracyModifier" => {item.percentaccuracymodifier = parsei16(reader, buf, &name);}
						b"usOverheatingCooldownFactor" => {item.usOverheatingCooldownFactor = parsef32(reader, buf, &name);}
						b"overheatTemperatureModificator" => {item.overheatTemperatureModificator = parsef32(reader, buf, &name);}
						b"overheatCooldownModificator" => {item.overheatCooldownModificator = parsef32(reader, buf, &name);}
						b"overheatJamThresholdModificator" => {item.overheatJamThresholdModificator = parsef32(reader, buf, &name);}
						b"overheatDamageThresholdModificator" => {item.overheatDamageThresholdModificator = parsef32(reader, buf, &name);}
						b"DirtIncreaseFactor" => {item.dirtIncreaseFactor = parsef32(reader, buf, &name);}
						b"RobotDamageReduction" => {item.fRobotDamageReductionModifier = parsef32(reader, buf, &name);}

						b"DiscardedLauncherItem" => {item.discardedlauncheritem = parseu16(reader, buf, &name);}
						b"randomitem" => {item.randomitem = parseu16(reader, buf, &name);}
						b"buddyitem" => {item.usBuddyItem = parseu16(reader, buf, &name);}
						b"usRiotShieldStrength" => {item.usRiotShieldStrength = parseu16(reader, buf, &name);}
						b"usRiotShieldGraphic" => {item.usRiotShieldGraphic = parseu16(reader, buf, &name);}

						b"PercentNoiseReduction" => {item.percentnoisereduction = parsei16(reader, buf, &name);}
						b"Bipod" => {item.bipod = parsei16(reader, buf, &name);}
						b"ToHitBonus" => {item.tohitbonus = parsei16(reader, buf, &name);}
						b"BestLaserRange" => {item.bestlaserrange = parsei16(reader, buf, &name);}
						b"RangeBonus" => {item.rangebonus = parsei16(reader, buf, &name);}
						b"PercentRangeBonus" => {item.percentrangebonus = parsei16(reader, buf, &name);}
						b"AimBonus" => {item.aimbonus = parsei16(reader, buf, &name);}
						b"MinRangeForAimBonus" => {item.minrangeforaimbonus = parsei16(reader, buf, &name);}
						b"PercentAPReduction" => {item.percentapreduction = parsei16(reader, buf, &name);}
						b"PercentStatusDrainReduction" => {item.percentstatusdrainreduction = parsei16(reader, buf, &name);}
						b"BloodiedItem" => {item.bloodieditem = parsei16(reader, buf, &name);}
						b"HearingRangeBonus" => {item.hearingrangebonus = parsei16(reader, buf, &name);}
						b"VisionRangeBonus" => {item.visionrangebonus = parsei16(reader, buf, &name);}
						b"NightVisionRangeBonus" => {item.nightvisionrangebonus = parsei16(reader, buf, &name);}
						b"DayVisionRangeBonus" => {item.dayvisionrangebonus = parsei16(reader, buf, &name);}
						b"CaveVisionRangeBonus" => {item.cavevisionrangebonus = parsei16(reader, buf, &name);}
						b"BrightLightVisionRangeBonus" => {item.brightlightvisionrangebonus = parsei16(reader, buf, &name);}
						b"DamageBonus" => {item.damagebonus = parsei16(reader, buf, &name);}
						b"MeleeDamageBonus" => {item.meleedamagebonus = parsei16(reader, buf, &name);}
						b"MagSizeBonus" => {item.magsizebonus = parsei16(reader, buf, &name);}
						b"PercentBurstFireAPReduction" => {item.percentburstfireapreduction = parsei16(reader, buf, &name);}
						b"PercentAutofireAPReduction" => {item.percentautofireapreduction = parsei16(reader, buf, &name);}
						b"PercentReadyTimeAPReduction" => {item.percentreadytimeapreduction = parsei16(reader, buf, &name);}
						b"PercentReloadTimeAPReduction" => {item.percentreloadtimeapreduction = parsei16(reader, buf, &name);}
		
						b"AutoFireToHitBonus" => {item.autofiretohitbonus = parsei16(reader, buf, &name);}
						b"APBonus" => {item.APBonus = parsei16(reader, buf, &name);}
						b"RateOfFireBonus" => {item.rateoffirebonus = parsei16(reader, buf, &name);}
						b"BurstSizeBonus" => {item.burstsizebonus = parsei16(reader, buf, &name);}
						b"BurstToHitBonus" => {item.bursttohitbonus = parsei16(reader, buf, &name);}
						b"BulletSpeedBonus" => {item.bulletspeedbonus = parsei16(reader, buf, &name);}
						b"CamoBonus" => {item.camobonus = parsei16(reader, buf, &name);}
						b"UrbanCamoBonus" => {item.urbanCamobonus = parsei16(reader, buf, &name);}
						b"DesertCamoBonus" => {item.desertCamobonus = parsei16(reader, buf, &name);}
						b"SnowCamoBonus" => {item.snowCamobonus = parsei16(reader, buf, &name);}
						b"StealthBonus" => {item.stealthbonus = parsei16(reader, buf, &name);}
						b"usSpotting" => {item.usSpotting = parsei16(reader, buf, &name);}
						b"sBackpackWeightModifier" => {item.sBackpackWeightModifier = parsei16(reader, buf, &name);}
						b"sFireResistance" => {item.sFireResistance = parsei16(reader, buf, &name);}
						b"PercentTunnelVision" => {item.percenttunnelvision = parseu8(reader, buf, &name);}
						b"AttachmentSystem" => {item.ubAttachmentSystem = parseu8(reader, buf, &name);}
						b"CrowbarModifier" => {item.CrowbarModifier = parseu8(reader, buf, &name);}
						b"DisarmModifier" => {item.DisarmModifier = parseu8(reader, buf, &name);}
						b"RepairModifier" => {item.RepairModifier = parsei8(reader, buf, &name);}
						b"usHackingModifier" => {item.usHackingModifier = parseu8(reader, buf, &name);}
						b"usBurialModifier" => {item.usBurialModifier = parseu8(reader, buf, &name);}
						b"DamageChance" => {item.usDamageChance = parseu8(reader, buf, &name);}
						b"FlashLightRange" => {item.usFlashLightRange = parseu8(reader, buf, &name);}
						b"ItemChoiceTimeSetting" => {item.usItemChoiceTimeSetting = parseu8(reader, buf, &name);}
						b"SleepModifier" => {item.ubSleepModifier = parseu8(reader, buf, &name);}
						b"usPortionSize" => {item.usPortionSize = parseu8(reader, buf, &name);}
						b"usAdministrationModifier" => { item.usAdministrationModifier = parseu8(reader, buf, &name); }
						b"Inseparable" => { item.inseparable = parseu8(reader, buf, &name); }
		
						b"LockPickModifier" => {item.LockPickModifier = parsei8(reader, buf, &name);}
						b"randomitemcoolnessmodificator" => {item.randomitemcoolnessmodificator = parsei8(reader, buf, &name);}
						b"RobotStrBonus" => {item.bRobotStrBonus = parsei8(reader, buf, &name);}
						b"RobotAgiBonus" => {item.bRobotAgiBonus = parsei8(reader, buf, &name);}
						b"RobotDexBonus" => {item.bRobotDexBonus = parsei8(reader, buf, &name);}
						b"RobotTargetingSkillGrant" => {item.bRobotTargetingSkillGrant = parsei8(reader, buf, &name);}
						b"RobotUtilitySkillGrant" => {item.bRobotUtilitySkillGrant = parsei8(reader, buf, &name);}
						b"RobotChassisSkillGrant" => {item.bRobotChassisSkillGrant = parsei8(reader, buf, &name);}
						b"TransportGroupMinProgress" => {item.iTransportGroupMinProgress = parsei8(reader, buf, &name);}
						b"TransportGroupMaxProgress" => {item.iTransportGroupMaxProgress = parsei8(reader, buf, &name);}

						b"BR_NewInventory" => {item.BR_NewInventory = parseu8(reader, buf, &name);}
						b"BR_UsedInventory" => {item.BR_UsedInventory = parseu8(reader, buf, &name);}
						b"BR_ROF" => {item.BR_ROF = parsei16(reader, buf, &name);}

						b"DefaultAttachment" => {
							let value = parseu16(reader, buf, &name);
							item.defaultattachments.push(value);
						}

						// STAND/CROUCH/PRONE_MODIFIERS
						b"STAND_MODIFIERS" => { self.readStanceModifiers(reader, 0); }
						b"CROUCH_MODIFIERS" => { self.readStanceModifiers(reader, 1); }
						b"PRONE_MODIFIERS" => { self.readStanceModifiers(reader, 2); }

						b"ItemFlag" => 
						{
							let readFlag = parseu64(reader, buf, &name); 
							// let i = self.items.last_mut().unwrap();

							for j in 0..63
							{
								if let Some(bit) = get_bit_at(readFlag, j) 
								{
									// Only set values that are not zero, so we don't accidentally override the already loaded old boolean tags
									if bit > 0 {
										item.usItemFlag = set_bit_at(item.usItemFlag, j, bit).unwrap();
									}
								}
							}
						}
						// Old booleans that should go into usItemFlag
						b"bloodbag" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 0, bit).unwrap();
						}
						b"emptybloodbag" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 32, bit).unwrap();
						}
						b"medicalsplint" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 33, bit).unwrap();
						}
						b"Damageable" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 34, bit).unwrap();
						}
						b"Repairable" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 35, bit).unwrap();
						}
						b"WaterDamages" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 36, bit).unwrap();
						}
						b"Metal" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 37, bit).unwrap();
						}
						b"Sinks" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 38, bit).unwrap();
						}
						b"ShowStatus" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 39, bit).unwrap();
						}
						b"HiddenAddon" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 40, bit).unwrap();
						}
						b"TwoHanded" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 41, bit).unwrap();
						}
						b"NotBuyable" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 42, bit).unwrap();
						}
						b"Attachment" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 43, bit).unwrap();
						}
						b"HiddenAttachment" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 44, bit).unwrap();
						}
						b"BigGunList" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 45, bit).unwrap();
						}
						b"NotInEditor" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 46, bit).unwrap();
						}
						b"DefaultUndroppable" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 47, bit).unwrap();
						}
						b"Unaerodynamic" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 48, bit).unwrap();
						}
						b"Electronic" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 49, bit).unwrap();
						}
						b"Cannon" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 50, bit).unwrap();
						}
						b"RocketRifle" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 51, bit).unwrap();
						}
						b"FingerPrintID" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 52, bit).unwrap();
						}
						b"MetalDetector" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 53, bit).unwrap();
						}
						b"GasMask" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 54, bit).unwrap();
						}
						b"LockBomb" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 55, bit).unwrap();
						}
						b"Flare" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 56, bit).unwrap();
						}
						b"GrenadeLauncher" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 57, bit).unwrap();
						}
						b"Mortar" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 58, bit).unwrap();
						}
						b"Duckbill" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 59, bit).unwrap();
						}
						b"Detonator" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 60, bit).unwrap();
						}
						b"RemoteDetonator" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 61, bit).unwrap();
						}
						b"HideMuzzleFlash" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 62, bit).unwrap();
						}
						b"RocketLauncher" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag = set_bit_at(item.usItemFlag, 63, bit).unwrap();
						}

						// ItemFlag2 tags
						b"SingleShotRocketLauncher" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 0, bit).unwrap();
						}
						b"BrassKnuckles" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 1, bit).unwrap();
						}
						b"Crowbar" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 2, bit).unwrap();
						}
						b"GLGrenade" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 3, bit).unwrap();
						}
						b"FlakJacket" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 4, bit).unwrap();
						}
						b"LeatherJacket" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 5, bit).unwrap();
						}
						b"Batteries" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 6, bit).unwrap();
						}
						b"NeedsBatteries" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 7, bit).unwrap();
						}
						b"XRay" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 8, bit).unwrap();
						}
						b"WireCutters" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 9, bit).unwrap();
						}
						b"Toolkit" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 10, bit).unwrap();
						}
						b"FirstAidKit" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 11, bit).unwrap();
						}
						b"MedicalKit" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 12, bit).unwrap();
						}
						b"Canteen" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 13, bit).unwrap();
						}
						b"Jar" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 14, bit).unwrap();
						}
						b"CanAndString" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 15, bit).unwrap();
						}
						b"Marbles" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 16, bit).unwrap();
						}
						b"Walkman" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 17, bit).unwrap();
						}
						b"RemoteTrigger" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 18, bit).unwrap();
						}
						b"RobotRemoteControl" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 19, bit).unwrap();
						}
						b"CamouflageKit" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 20, bit).unwrap();
						}
						b"LocksmithKit" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 21, bit).unwrap();
						}
						b"Mine" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 22, bit).unwrap();
						}
						b"antitankmine" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 23, bit).unwrap();
						}
						b"Hardware" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 24, bit).unwrap();
						}
						b"Medical" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 25, bit).unwrap();
						}
						b"GasCan" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 26, bit).unwrap();
						}
						b"ContainsLiquid" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 27, bit).unwrap();
						}
						b"Rock" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 28, bit).unwrap();
						}
						b"ThermalOptics" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 29, bit).unwrap();
						}
						b"SciFi" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 30, bit).unwrap();
						}
						b"NewInv" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 31, bit).unwrap();
						}
						b"DiseaseSystemExclusive" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 32, bit).unwrap();
						}
						b"barrel" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 33, bit).unwrap();
						}
						b"TripWireActivation" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 34, bit).unwrap();
						}
						b"TripWire" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 35, bit).unwrap();
						}
						b"Directional" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 36, bit).unwrap();
						}
						b"BlockIronSight" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 37, bit).unwrap();
						}
						b"fAllowClimbing" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 38, bit).unwrap();
						}
						b"cigarette" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 39, bit).unwrap();
						}
						b"ProvidesRobotCamo" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 40, bit).unwrap();
						}
						b"ProvidesRobotNightVision" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 41, bit).unwrap();
						}
						b"ProvidesRobotLaserBonus" => 
						{
							let bit = parseu8(reader, buf, &name);
							item.usItemFlag2 = set_bit_at(item.usItemFlag2, 42, bit).unwrap();
						}

						_ => {}
					}
				}

				Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
				Ok(Event::End(ref element)) => 
				{
					match element.name().as_ref()
					{
						b"ITEM" => break,
						_ => ()
					}
				}
				_ => (),
			}
			buf.clear();
		}	
	}

	fn readStanceModifiers(&mut self, reader: &mut Reader<BufReader<File>>, i: usize)
	{
		let mut buf = Vec::new();
		loop {
			match reader.read_event_into(&mut buf) 
			{
				Ok(Event::Start(e)) => 
				{
					let name = str::from_utf8(e.name().as_ref()).unwrap().to_string();
					match e.name().as_ref()
					{
						b"FlatBase" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().flatbasemodifier[i] = value;
						}
						b"PercentBase" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().percentbasemodifier[i] = value;
						}
						b"FlatAim" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().flataimmodifier[i] = value;
						}
						b"PercentAim" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().percentaimmodifier[i] = value;
						}
						b"PercentCap" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().percentcapmodifier[i] = value;
						}
						b"PercentHandling" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().percenthandlingmodifier[i] = value;
						}
						b"PercentTargetTrackingSpeed" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().targettrackingmodifier[i] = value;
						}
						b"PercentDropCompensation" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().percentdropcompensationmodifier[i] = value;
						}
						b"PercentMaxCounterForce" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().maxcounterforcemodifier[i] = value;
						}
						b"PercentCounterForceAccuracy" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().counterforceaccuracymodifier[i] = value;
						}
						b"AimLevels" => {
							let value = parsei16(reader, &mut buf, &name);
							self.items.last_mut().unwrap().aimlevelsmodifier[i] = value;
						}
						_ => ()
					}
				}
				Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
				Ok(Event::End(ref element)) => 
				{
					match element.name().as_ref()
					{
						b"STAND_MODIFIERS" => break,
						b"CROUCH_MODIFIERS" => break,
						b"PRONE_MODIFIERS" => break,
						_ => ()
					}
				}
				_ => ()
			}
			buf.clear();
		}
	}

	fn saveItems(&self, filepath: &PathBuf)
    {
        let mut buffer = Vec::new();
        // Write xml header before the xml data
        write!(buffer, "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n").unwrap();

		write!(buffer, "<ITEMLIST>\n").unwrap();

		let mut forcewriteFirst = true;
        for i in &self.items
        {
	    	write!(buffer, "\t<ITEM>\n").unwrap();

            let value = i.uiIndex;
            write_tag_i!(buffer, value, "uiIndex", forcewriteFirst);
            
            let value = &i.szItemName;
            write_tag_s!(buffer, value, "szItemName", forcewriteFirst);

			let value = &i.szLongItemName;
            write_tag_s!(buffer, value, "szLongItemName", forcewriteFirst);

			let value = &i.szItemDesc;
            write_tag_s!(buffer, value, "szItemDesc", forcewriteFirst);

			let value = &i.szBRName;
            write_tag_s!(buffer, value, "szBRName", forcewriteFirst);

			let value = &i.szBRDesc;
            write_tag_s!(buffer, value, "szBRDesc", forcewriteFirst);

            let value = i.usItemClass;
            write_tag_i!(buffer, value, "usItemClass", forcewriteFirst);

			let value = i.attachmentclass;
			write_tag_i!(buffer, value, "AttachmentClass", forcewriteFirst);

			let value = i.nasAttachmentClass;
			write_tag_i!(buffer, value, "nasAttachmentClass", forcewriteFirst);

			let value = i.nasLayoutClass;
			write_tag_i!(buffer, value, "nasLayoutClass", forcewriteFirst);

			let value = i.ulAvailableAttachmentPoint;
			write_tag_i!(buffer, value, "AvailableAttachmentPoint", forcewriteFirst);

			let value = i.ulAttachmentPoint;
			write_tag_i!(buffer, value, "AttachmentPoint", forcewriteFirst);

			let value = i.ubAttachToPointAPCost;
			write_tag_i!(buffer, value, "AttachToPointAPCost", forcewriteFirst);

			let value = i.ubClassIndex;
			write_tag_i!(buffer, value, "ubClassIndex", forcewriteFirst);

			let value = i.ubCursor;
			write_tag_i!(buffer, value, "ubCursor", forcewriteFirst);

			let value = i.bSoundType;
			write_tag_i!(buffer, value, "bSoundType", forcewriteFirst);

			let value = i.ubGraphicType;
			write_tag_i!(buffer, value, "ubGraphicType", forcewriteFirst);

			let value = i.ubGraphicNum;
			write_tag_i!(buffer, value, "ubGraphicNum", forcewriteFirst);

			let value = i.ubWeight;
			write_tag_i!(buffer, value, "ubWeight", forcewriteFirst);
	
			let value = i.ubPerPocket;
			write_tag_i!(buffer, value, "ubPerPocket", forcewriteFirst);
	
			let value = i.ItemSize;
			write_tag_i!(buffer, value, "ItemSize", forcewriteFirst);
	
			let value = i.itemsizebonus;
			write_tag_i!(buffer, value, "ItemSizeBonus", forcewriteFirst);

			let value = i.usPrice;
			write_tag_i!(buffer, value, "usPrice", forcewriteFirst);
	
			let value = i.ubCoolness;
			write_tag_i!(buffer, value, "ubCoolness", forcewriteFirst);
	
			let value = i.bReliability;
			write_tag_i!(buffer, value, "bReliability", forcewriteFirst);
	
			let value = i.bRepairEase;
			write_tag_i!(buffer, value, "bRepairEase", forcewriteFirst);

			let value = i.drugtype;
			write_tag_i!(buffer, value, "DrugType", forcewriteFirst);
	
			let value = i.usActionItemFlag;
			write_tag_i!(buffer, value, "usActionItemFlag", forcewriteFirst);

			let value = i.clothestype;
			write_tag_i!(buffer, value, "clothestype", forcewriteFirst);
			
            let value = &i.spreadPattern;
            write_tag_s!(buffer, value, "spreadPattern", forcewriteFirst);

			let value = i.alcohol;
            write_tag_f!(buffer, value, "Alcohol", forcewriteFirst);

			let value = i.scopemagfactor;
            write_tag_f!(buffer, value, "ScopeMagFactor", forcewriteFirst);

			let value = i.projectionfactor;
            write_tag_f!(buffer, value, "ProjectionFactor", forcewriteFirst);

			let value = i.RecoilModifierX;
            write_tag_f!(buffer, value, "RecoilModifierX", forcewriteFirst);

			let value = i.RecoilModifierY;
            write_tag_f!(buffer, value, "RecoilModifierY", forcewriteFirst);

			let value = i.PercentRecoilModifier;
            write_tag_i!(buffer, value, "PercentRecoilModifier", forcewriteFirst);

			let value = i.percentaccuracymodifier;
            write_tag_i!(buffer, value, "PercentAccuracyModifier", forcewriteFirst);

			let value = i.usOverheatingCooldownFactor;
            write_tag_f!(buffer, value, "usOverheatingCooldownFactor", forcewriteFirst);

			let value = i.overheatTemperatureModificator;
            write_tag_f!(buffer, value, "overheatTemperatureModificator", forcewriteFirst);

			let value = i.overheatCooldownModificator;
            write_tag_f!(buffer, value, "overheatCooldownModificator", forcewriteFirst);

			let value = i.overheatJamThresholdModificator;
            write_tag_f!(buffer, value, "overheatJamThresholdModificator", forcewriteFirst);

			let value = i.overheatDamageThresholdModificator;
            write_tag_f!(buffer, value, "overheatDamageThresholdModificator", forcewriteFirst);

			let value = i.dirtIncreaseFactor;
            write_tag_f!(buffer, value, "DirtIncreaseFactor", forcewriteFirst);

			let value = i.fRobotDamageReductionModifier;
            write_tag_f!(buffer, value, "RobotDamageReduction", forcewriteFirst);

			let value = i.discardedlauncheritem;
			write_tag_i!(buffer, value, "DiscardedLauncherItem", forcewriteFirst);

			let value = i.randomitem;
			write_tag_i!(buffer, value, "randomitem", forcewriteFirst);

			let value = i.usBuddyItem;
			write_tag_i!(buffer, value, "buddyitem", forcewriteFirst);

			let value = i.usRiotShieldStrength;
			write_tag_i!(buffer, value, "usRiotShieldStrength", forcewriteFirst);

			let value = i.usRiotShieldGraphic;
			write_tag_i!(buffer, value, "usRiotShieldGraphic", forcewriteFirst);

			let value = i.percentnoisereduction;
			write_tag_i!(buffer, value, "PercentNoiseReduction", forcewriteFirst);

			let value = i.bipod;
			write_tag_i!(buffer, value, "Bipod", forcewriteFirst);

			let value = i.tohitbonus;
			write_tag_i!(buffer, value, "ToHitBonus", forcewriteFirst);

			let value = i.bestlaserrange;
			write_tag_i!(buffer, value, "BestLaserRange", forcewriteFirst);

			let value = i.rangebonus;
			write_tag_i!(buffer, value, "RangeBonus", forcewriteFirst);

			let value = i.percentrangebonus;
			write_tag_i!(buffer, value, "PercentRangeBonus", forcewriteFirst);

			let value = i.aimbonus;
			write_tag_i!(buffer, value, "AimBonus", forcewriteFirst);

			let value = i.minrangeforaimbonus;
			write_tag_i!(buffer, value, "MinRangeForAimBonus", forcewriteFirst);

			let value = i.percentapreduction;
			write_tag_i!(buffer, value, "PercentAPReduction", forcewriteFirst);

			let value = i.percentstatusdrainreduction;
			write_tag_i!(buffer, value, "PercentStatusDrainReduction", forcewriteFirst);

			let value = i.bloodieditem;
			write_tag_i!(buffer, value, "BloodiedItem", forcewriteFirst);

			let value = i.hearingrangebonus;
			write_tag_i!(buffer, value, "HearingRangeBonus", forcewriteFirst);

			let value = i.visionrangebonus;
			write_tag_i!(buffer, value, "VisionRangeBonus", forcewriteFirst);

			let value = i.nightvisionrangebonus;
			write_tag_i!(buffer, value, "NightVisionRangeBonus", forcewriteFirst);

			let value = i.dayvisionrangebonus;
			write_tag_i!(buffer, value, "DayVisionRangeBonus", forcewriteFirst);

			let value = i.cavevisionrangebonus;
			write_tag_i!(buffer, value, "CaveVisionRangeBonus", forcewriteFirst);

			let value = i.brightlightvisionrangebonus;
			write_tag_i!(buffer, value, "BrightLightVisionRangeBonus", forcewriteFirst);

			let value = i.damagebonus;
			write_tag_i!(buffer, value, "DamageBonus", forcewriteFirst);

			let value = i.meleedamagebonus;
			write_tag_i!(buffer, value, "MeleeDamageBonus", forcewriteFirst);

			let value = i.magsizebonus;
			write_tag_i!(buffer, value, "MagSizeBonus", forcewriteFirst);

			let value = i.percentburstfireapreduction;
			write_tag_i!(buffer, value, "PercentBurstFireAPReduction", forcewriteFirst);

			let value = i.percentautofireapreduction;
			write_tag_i!(buffer, value, "PercentAutofireAPReduction", forcewriteFirst);

			let value = i.percentreadytimeapreduction;
			write_tag_i!(buffer, value, "PercentReadyTimeAPReduction", forcewriteFirst);

			let value = i.percentreloadtimeapreduction;
			write_tag_i!(buffer, value, "PercentReloadTimeAPReduction", forcewriteFirst);

			let value = i.autofiretohitbonus;
			write_tag_i!(buffer, value, "AutoFireToHitBonus", forcewriteFirst);

			let value = i.APBonus;
			write_tag_i!(buffer, value, "APBonus", forcewriteFirst);

			let value = i.rateoffirebonus;
			write_tag_i!(buffer, value, "RateOfFireBonus", forcewriteFirst);

			let value = i.burstsizebonus;
			write_tag_i!(buffer, value, "BurstSizeBonus", forcewriteFirst);

			let value = i.bursttohitbonus;
			write_tag_i!(buffer, value, "BurstToHitBonus", forcewriteFirst);

			let value = i.bulletspeedbonus;
			write_tag_i!(buffer, value, "BulletSpeedBonus", forcewriteFirst);

			let value = i.camobonus;
			write_tag_i!(buffer, value, "CamoBonus", forcewriteFirst);

			let value = i.urbanCamobonus;
			write_tag_i!(buffer, value, "UrbanCamoBonus", forcewriteFirst);

			let value = i.desertCamobonus;
			write_tag_i!(buffer, value, "DesertCamoBonus", forcewriteFirst);

			let value = i.snowCamobonus;
			write_tag_i!(buffer, value, "SnowCamoBonus", forcewriteFirst);

			let value = i.stealthbonus;
			write_tag_i!(buffer, value, "StealthBonus", forcewriteFirst);

			let value = i.usSpotting;
			write_tag_i!(buffer, value, "usSpotting", forcewriteFirst);

			let value = i.sBackpackWeightModifier;
			write_tag_i!(buffer, value, "sBackpackWeightModifier", forcewriteFirst);

			let value = i.sFireResistance;
			write_tag_i!(buffer, value, "sFireResistance", forcewriteFirst);

			let value = i.percenttunnelvision;
			write_tag_i!(buffer, value, "PercentTunnelVision", forcewriteFirst);

			let value = i.ubAttachmentSystem;
			write_tag_i!(buffer, value, "AttachmentSystem", forcewriteFirst);

			let value = i.CrowbarModifier;
			write_tag_i!(buffer, value, "CrowbarModifier", forcewriteFirst);

			let value = i.DisarmModifier;
			write_tag_i!(buffer, value, "DisarmModifier", forcewriteFirst);

			let value = i.RepairModifier;
			write_tag_i!(buffer, value, "RepairModifier", forcewriteFirst);

			let value = i.usHackingModifier;
			write_tag_i!(buffer, value, "usHackingModifier", forcewriteFirst);

			let value = i.usBurialModifier;
			write_tag_i!(buffer, value, "usBurialModifier", forcewriteFirst);

			let value = i.usDamageChance;
			write_tag_i!(buffer, value, "DamageChance", forcewriteFirst);

			let value = i.usFlashLightRange;
			write_tag_i!(buffer, value, "FlashLightRange", forcewriteFirst);

			let value = i.usItemChoiceTimeSetting;
			write_tag_i!(buffer, value, "ItemChoiceTimeSetting", forcewriteFirst);

			let value = i.ubSleepModifier;
			write_tag_i!(buffer, value, "SleepModifier", forcewriteFirst);

			let value = i.usPortionSize;
			write_tag_i!(buffer, value, "usPortionSize", forcewriteFirst);

			let value = i.usAdministrationModifier;
			write_tag_i!(buffer, value, "usAdministrationModifier", forcewriteFirst);

			let value = i.inseparable;
			write_tag_i!(buffer, value, "Inseparable", forcewriteFirst);

			let value = i.LockPickModifier;
			write_tag_i!(buffer, value, "LockPickModifier", forcewriteFirst);

			let value = i.randomitemcoolnessmodificator;
			write_tag_i!(buffer, value, "randomitemcoolnessmodificator", forcewriteFirst);

			let value = i.bRobotStrBonus;
			write_tag_i!(buffer, value, "RobotStrBonus", forcewriteFirst);

			let value = i.bRobotAgiBonus;
			write_tag_i!(buffer, value, "RobotAgiBonus", forcewriteFirst);

			let value = i.bRobotDexBonus;
			write_tag_i!(buffer, value, "RobotDexBonus", forcewriteFirst);

			let value = i.bRobotTargetingSkillGrant;
			write_tag_i!(buffer, value, "RobotTargetingSkillGrant", forcewriteFirst);

			let value = i.bRobotUtilitySkillGrant;
			write_tag_i!(buffer, value, "RobotUtilitySkillGrant", forcewriteFirst);

			let value = i.bRobotChassisSkillGrant;
			write_tag_i!(buffer, value, "RobotChassisSkillGrant", forcewriteFirst);

			let value = i.iTransportGroupMinProgress;
			write_tag_i!(buffer, value, "TransportGroupMinProgress", forcewriteFirst);

			let value = i.iTransportGroupMaxProgress;
			write_tag_i!(buffer, value, "TransportGroupMaxProgress", forcewriteFirst);

			let value = i.BR_NewInventory;
			write_tag_i!(buffer, value, "BR_NewInventory", forcewriteFirst);

			let value = i.BR_UsedInventory;
			write_tag_i!(buffer, value, "BR_UsedInventory", forcewriteFirst);

			let value = i.BR_ROF;
			write_tag_i!(buffer, value, "BR_ROF", forcewriteFirst);

			for p in &i.defaultattachments
			{
				let p = p.clone();
				write_tag_i!(buffer, p, "DefaultAttachment", forcewriteFirst);
			}
	
			// Itemflag
			for j in 0..64
			{
		        if let Some(bit) = get_bit_at(i.usItemFlag, j) 
				{
					let value = bit;

					match j 
					{
						0 => { write_tag_i!(buffer, value, "Bloodbag", forcewriteFirst); },
						1 => { write_tag_i!(buffer, value, "Manpad", forcewriteFirst); },
						2 => { write_tag_i!(buffer, value, "Beartrap", forcewriteFirst); },
						3 => { write_tag_i!(buffer, value, "Camera", forcewriteFirst); },
						4 => { write_tag_i!(buffer, value, "Waterdrum", forcewriteFirst); },
						5 => { write_tag_i!(buffer, value, "BloodcatMeat", forcewriteFirst); },
						6 => { write_tag_i!(buffer, value, "CowMeat", forcewriteFirst); },
						7 => { write_tag_i!(buffer, value, "Beltfed", forcewriteFirst); },
						8 => { write_tag_i!(buffer, value, "Ammobelt", forcewriteFirst); },
						9 => { write_tag_i!(buffer, value, "AmmobeltVest", forcewriteFirst); },

						10 => { write_tag_i!(buffer, value, "CamoRemoval", forcewriteFirst); },
						11 => { write_tag_i!(buffer, value, "Cleaningkit", forcewriteFirst); },
						12 => { write_tag_i!(buffer, value, "AttentionItem", forcewriteFirst); },
						13 => { write_tag_i!(buffer, value, "Garotte", forcewriteFirst); },
						14 => { write_tag_i!(buffer, value, "Covert", forcewriteFirst); },
						15 => { write_tag_i!(buffer, value, "Corpse", forcewriteFirst); },
						16 => { write_tag_i!(buffer, value, "BloodcatSkin", forcewriteFirst); },
						17 => { write_tag_i!(buffer, value, "NoMetalDetection", forcewriteFirst); },
						18 => { write_tag_i!(buffer, value, "JumpGrenade", forcewriteFirst); },
						19 => { write_tag_i!(buffer, value, "Handcuffs", forcewriteFirst); },
						
						20 => { write_tag_i!(buffer, value, "Taser", forcewriteFirst); },
						21 => { write_tag_i!(buffer, value, "ScubaBottle", forcewriteFirst); },
						22 => { write_tag_i!(buffer, value, "ScubaMask", forcewriteFirst); },
						23 => { write_tag_i!(buffer, value, "ScubaFins", forcewriteFirst); },
						24 => { write_tag_i!(buffer, value, "TripwireRoll", forcewriteFirst); },
						25 => { write_tag_i!(buffer, value, "Radioset", forcewriteFirst); },
						26 => { write_tag_i!(buffer, value, "SignalShell", forcewriteFirst); },
						27 => { write_tag_i!(buffer, value, "Soda", forcewriteFirst); },
						28 => { write_tag_i!(buffer, value, "RoofcollapseItem", forcewriteFirst); },
						29 => { write_tag_i!(buffer, value, "DiseaseprotectionFace", forcewriteFirst); },
						
						30 => { write_tag_i!(buffer, value, "DiseaseprotectionHand", forcewriteFirst); },
						31 => { write_tag_i!(buffer, value, "LBEexplosionproof", forcewriteFirst); },
						32 => { write_tag_i!(buffer, value, "EmptyBloodbag", forcewriteFirst); },
						33 => { write_tag_i!(buffer, value, "MedicalSplint", forcewriteFirst); },
						34 => { write_tag_i!(buffer, value, "Damageable", forcewriteFirst); },
						35 => { write_tag_i!(buffer, value, "Repairable", forcewriteFirst); },
						36 => { write_tag_i!(buffer, value, "WaterDamages", forcewriteFirst); },
						37 => { write_tag_i!(buffer, value, "Metal", forcewriteFirst); },
						38 => { write_tag_i!(buffer, value, "Sinks", forcewriteFirst); },
						39 => { write_tag_i!(buffer, value, "ShowStatus", forcewriteFirst); },

						40 => { write_tag_i!(buffer, value, "HiddenAddon", forcewriteFirst); },
						41 => { write_tag_i!(buffer, value, "TwoHanded", forcewriteFirst); },
						42 => { write_tag_i!(buffer, value, "NotBuyable", forcewriteFirst); },
						43 => { write_tag_i!(buffer, value, "Attachment", forcewriteFirst); },
						44 => { write_tag_i!(buffer, value, "HiddenAttachment", forcewriteFirst); },
						45 => { write_tag_i!(buffer, value, "BigGunList", forcewriteFirst); },
						46 => { write_tag_i!(buffer, value, "NotInEditor", forcewriteFirst); },
						47 => { write_tag_i!(buffer, value, "DefaultUndroppable", forcewriteFirst); },
						48 => { write_tag_i!(buffer, value, "Unaerodynamic", forcewriteFirst); },
						49 => { write_tag_i!(buffer, value, "Electronic", forcewriteFirst); },

						50 => { write_tag_i!(buffer, value, "Cannon", forcewriteFirst); },
						51 => { write_tag_i!(buffer, value, "RocketRifle", forcewriteFirst); },
						52 => { write_tag_i!(buffer, value, "FingerPrintID", forcewriteFirst); },
						53 => { write_tag_i!(buffer, value, "MetalDetector", forcewriteFirst); },
						54 => { write_tag_i!(buffer, value, "GasMask", forcewriteFirst); },
						55 => { write_tag_i!(buffer, value, "LockBomb", forcewriteFirst); },
						56 => { write_tag_i!(buffer, value, "Flare", forcewriteFirst); },
						57 => { write_tag_i!(buffer, value, "GrenadeLauncher", forcewriteFirst); },
						58 => { write_tag_i!(buffer, value, "Mortar", forcewriteFirst); },
						59 => { write_tag_i!(buffer, value, "Duckbill", forcewriteFirst); },
						
						60 => { write_tag_i!(buffer, value, "Detonator", forcewriteFirst); },
						61 => { write_tag_i!(buffer, value, "RemoteDetonator", forcewriteFirst); },
						62 => { write_tag_i!(buffer, value, "HideMuzzleFlash", forcewriteFirst); },
						63 => { write_tag_i!(buffer, value, "RocketLauncher", forcewriteFirst); },
						_ => {}
					}
				}
			}

			// Itemflag2
			for j in 0..64
			{
				if let Some(bit) = get_bit_at(i.usItemFlag2, j) 
				{
					let value = bit;

					match j 
					{
						0 => { write_tag_i!(buffer, value, "SingleShotRocketLauncher", forcewriteFirst); },
						1 => { write_tag_i!(buffer, value, "BrassKnuckles", forcewriteFirst); },
						2 => { write_tag_i!(buffer, value, "Crowbar", forcewriteFirst); },
						3 => { write_tag_i!(buffer, value, "GLGrenade", forcewriteFirst); },
						4 => { write_tag_i!(buffer, value, "FlakJacket", forcewriteFirst); },
						5 => { write_tag_i!(buffer, value, "LeatherJacket", forcewriteFirst); },
						6 => { write_tag_i!(buffer, value, "Batteries", forcewriteFirst); },
						7 => { write_tag_i!(buffer, value, "NeedsBatteries", forcewriteFirst); },
						8 => { write_tag_i!(buffer, value, "XRay", forcewriteFirst); },
						9 => { write_tag_i!(buffer, value, "WireCutters", forcewriteFirst); },

						10 => { write_tag_i!(buffer, value, "Toolkit", forcewriteFirst); },
						11 => { write_tag_i!(buffer, value, "FirstAidKit", forcewriteFirst); },
						12 => { write_tag_i!(buffer, value, "MedicalKit", forcewriteFirst); },
						13 => { write_tag_i!(buffer, value, "Canteen", forcewriteFirst); },
						14 => { write_tag_i!(buffer, value, "Jar", forcewriteFirst); },
						15 => { write_tag_i!(buffer, value, "CanAndString", forcewriteFirst); },
						16 => { write_tag_i!(buffer, value, "Marbles", forcewriteFirst); },
						17 => { write_tag_i!(buffer, value, "Walkman", forcewriteFirst); },
						18 => { write_tag_i!(buffer, value, "RemoteTrigger", forcewriteFirst); },
						19 => { write_tag_i!(buffer, value, "RobotRemoteControl", forcewriteFirst); },
						
						20 => { write_tag_i!(buffer, value, "CamouflageKit", forcewriteFirst); },
						21 => { write_tag_i!(buffer, value, "LocksmithKit", forcewriteFirst); },
						22 => { write_tag_i!(buffer, value, "Mine", forcewriteFirst); },
						23 => { write_tag_i!(buffer, value, "AntitankMine", forcewriteFirst); },
						24 => { write_tag_i!(buffer, value, "Hardware", forcewriteFirst); },
						25 => { write_tag_i!(buffer, value, "Medical", forcewriteFirst); },
						26 => { write_tag_i!(buffer, value, "GasCan", forcewriteFirst); },
						27 => { write_tag_i!(buffer, value, "ContainsLiquid", forcewriteFirst); },
						28 => { write_tag_i!(buffer, value, "Rock", forcewriteFirst); },
						29 => { write_tag_i!(buffer, value, "ThermalOptics", forcewriteFirst); },
						
						30 => { write_tag_i!(buffer, value, "SciFi", forcewriteFirst); },
						31 => { write_tag_i!(buffer, value, "NewInv", forcewriteFirst); },
						32 => { write_tag_i!(buffer, value, "DiseaseSystemExclusive", forcewriteFirst); },
						33 => { write_tag_i!(buffer, value, "Barrel", forcewriteFirst); },
						34 => { write_tag_i!(buffer, value, "TripWireActivation", forcewriteFirst); },
						35 => { write_tag_i!(buffer, value, "TripWire", forcewriteFirst); },
						36 => { write_tag_i!(buffer, value, "Directional", forcewriteFirst); },
						37 => { write_tag_i!(buffer, value, "BlockIronSight", forcewriteFirst); },
						38 => { write_tag_i!(buffer, value, "AllowClimbing", forcewriteFirst); },
						39 => { write_tag_i!(buffer, value, "Cigarette", forcewriteFirst); },
						
						40 => { write_tag_i!(buffer, value, "ProvidesRobotCamo", forcewriteFirst); },
						41 => { write_tag_i!(buffer, value, "ProvidesRobotNightVision", forcewriteFirst); },
						42 => { write_tag_i!(buffer, value, "ProvidesRobotLaserBonus", forcewriteFirst); },
						// 43 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 44 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 45 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 46 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 47 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 48 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 49 => { write_tag_i!(buffer, value, "", forcewriteFirst); },

						// 50 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 51 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 52 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 53 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 54 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 55 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 56 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 57 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 58 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 59 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						
						// 60 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 61 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 62 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						// 63 => { write_tag_i!(buffer, value, "", forcewriteFirst); },
						_ => {}
					}
				}
			}

			let s = ["STAND_MODIFIERS", "CROUCH_MODIFIERS", "PRONE_MODIFIERS"];
			for j in 0..3
			{
				let flatbase = i.flatbasemodifier[j];
				let percentbase = i.percentbasemodifier[j];
				let flataim = i.flataimmodifier[j];
				let percentcap = i.percentcapmodifier[j];
				let percenthandling = i.percenthandlingmodifier[j];
				let targettracking = i.targettrackingmodifier[j];
				let dropcompensation = i.percentdropcompensationmodifier[j];
				let maxcounterforce = i.maxcounterforcemodifier[j];
				let counterforceaccuracy = i.counterforceaccuracymodifier[j];
				let aimlevels = i.aimlevelsmodifier[j];
	
				if forcewriteFirst == false && flatbase == 0 && percentbase == 0 && flataim == 0 && percentcap == 0 && percenthandling == 0 && targettracking == 0 && dropcompensation == 0 && maxcounterforce == 0 && counterforceaccuracy == 0 && aimlevels == 0
				{
					write!(buffer, "\t\t<{} />\n", s[j]).unwrap();
				}
				else
				{
					write!(buffer, "\t\t<{}>\n", s[j]).unwrap();
	
					if flatbase != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, flatbase, "FlatBase", forcewriteFirst);
					}
					if percentbase != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, percentbase, "PercentBase", forcewriteFirst);
					}
					if flataim != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, flataim, "FlatAim", forcewriteFirst);
					}
					if percentcap != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, percentcap, "PercentCap", forcewriteFirst);
					}
					if percenthandling != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, percenthandling, "PercentHandling", forcewriteFirst);
					}
					if targettracking != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, targettracking, "PercentTargetTrackingSpeed", forcewriteFirst);
					}
					if dropcompensation != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, dropcompensation, "PercentDropCompensation", forcewriteFirst);
					}
					if maxcounterforce != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, maxcounterforce, "PercentMaxCounterForce", forcewriteFirst);
					}
					if counterforceaccuracy != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, counterforceaccuracy, "PercentCounterForceAccuracy", forcewriteFirst);
					}
					if aimlevels != 0 || forcewriteFirst == true {
						write!(buffer, "\t").unwrap();
						write_tag_i!(buffer, aimlevels, "AimLevels", forcewriteFirst);
					}
	
					write!(buffer, "\t\t</{}>\n", s[j]).unwrap();
				}
			}
	

            write!(buffer, "\t</ITEM>\n").unwrap();

			forcewriteFirst = false;
        }


		write!(buffer, "</ITEMLIST>\n").unwrap();

        println!("{}", &filepath.to_str().unwrap());
        std::fs::create_dir_all(filepath.parent().unwrap());
        let mut file = File::create(filepath).unwrap();
        file.write_all(&buffer);
    }

}
//-----------------------------------------------------------------------------
// Functions
//-----------------------------------------------------------------------------
fn set_bit_at(x: u64, index: u8, bit: u8) -> Option<u64> {
    if index >= 64 {
        println!("The new bit is out of u64 range.");
        println!("0b_11111111");
        println!("  ^ trying to set here");
        return None;
    }

    if bit == 0
    {
        let bitmask = 1 << index;
        let bitmask_flipped = !bitmask; // flip all bits
        let result = x & bitmask_flipped;
        // println!("x        = {},    binary = {:#010b} ", x, x);
        // println!("bitmask  = {},    binary = {:#010b} ", bitmask, bitmask);
        // println!("!bitmask = {},    binary = {:#010b} ", bitmask_flipped, bitmask_flipped);
        // println!("result   = {},    binary = {:#010b} ", result, result);
        Some(result)
    }
    else
    {
        let bitmask = 1 << index;
        let result = x | bitmask;
        // println!("x        = {},    binary = {:#010b} ", x, x);
        // println!("bitmask  = {},    binary = {:#010b} ", bitmask, bitmask);
        // println!("result   = {},    binary = {:#010b} ", result, result);
        Some(result)

    }
}


fn get_bit_at(x: u64, index: u8) -> Option<u8> {
    if index >= 64
    {
        println!("Index is out of u64 range.");
        println!("0b_11111111");
        println!("  ^ trying to read from here");
        return None;
    }
    if x & (1 << index) != 0 {return Some(1);}
    else {return Some(0);}
}


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

