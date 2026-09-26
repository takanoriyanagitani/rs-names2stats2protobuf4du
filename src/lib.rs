use std::collections::HashMap;
use std::fs::Metadata;
use std::io;

use io::BufWriter;
use io::Write;

use io::BufRead;

use std::os::darwin;
use std::os::unix;

use protobuf::Message;

use protobuf::well_known_types::struct_::Struct;
use protobuf::well_known_types::struct_::Value;

/// Metadata useful for disk usage inspection(protobuf friendly).
#[derive(Default, Debug, Clone)]
pub struct DuInfoDTO(pub Struct);

impl DuInfoDTO {
    pub fn clear(&mut self) {
        self.0.fields.clear()
    }
}

pub fn string2value(s: String) -> Value {
    let mut v: Value = Value::default();
    v.set_string_value(s);
    v
}

pub fn double2value(d: f64) -> Value {
    let mut v: Value = Value::new();
    v.set_number_value(d);
    v
}

pub fn i2value(i: i32) -> Value {
    let f: f64 = i.into();
    double2value(f)
}

pub fn b2value(b: bool) -> Value {
    let mut v: Value = Value::default();
    v.set_bool_value(b);
    v
}

impl DuInfoDTO {
    pub fn import_filename(&mut self, filename: String) {
        self.0
            .fields
            .insert("filename".into(), string2value(filename));
    }
}

impl DuInfoDTO {
    pub fn import_not_found(&mut self, filename: String, errno: Option<i32>) {
        self.clear();
        self.import_filename(filename);
        self.0.fields.insert("not_found".into(), b2value(true));
        if let Some(eno) = errno {
            self.0.fields.insert("raw_os_error".into(), i2value(eno));
        }
    }

    pub fn import_other_err(&mut self, filename: String, errno: Option<i32>) {
        self.clear();
        self.import_filename(filename);
        if let Some(eno) = errno {
            self.0.fields.insert("raw_os_error".into(), i2value(eno));
        }
    }
}

impl DuInfoDTO {
    pub fn as_map(&self) -> &HashMap<String, Value> {
        &self.0.fields
    }
}

pub fn ulong2double(ulng: u64) -> Result<f64, io::Error> {
    let converted: f64 = ulng as f64;
    let check: u64 = converted as u64;
    match check == ulng {
        true => Ok(converted),
        false => Err(io::Error::other("unable to convert the ulong")),
    }
}

pub fn long2double(lng: i64) -> Result<f64, io::Error> {
    let converted: f64 = lng as f64;
    let check: i64 = converted as i64;
    match check == lng {
        true => Ok(converted),
        false => Err(io::Error::other("unable to convert the long")),
    }
}

impl DuInfoDTO {
    pub fn upsert_double(&mut self, key: &str, val: f64) {
        self.0
            .fields
            .entry(key.into())
            .and_modify(|v| *v = double2value(val))
            .or_insert(double2value(val));
    }
}

#[cfg(target_family = "unix")]
impl DuInfoDTO {
    pub fn import_metadata4unix<M>(&mut self, m: &M) -> Result<(), io::Error>
    where
        M: unix::fs::MetadataExt,
    {
        let dev: u64 = m.dev();
        let ino: u64 = m.ino();
        let mode: u32 = m.mode();
        let nlink: u64 = m.nlink();
        let uid: u32 = m.uid();
        let gid: u32 = m.gid();
        let rdev: u64 = m.rdev();
        let size: u64 = m.size();
        let atime: i64 = m.atime();
        let mtime: i64 = m.mtime();
        let ctime: i64 = m.ctime();
        let blksize: u64 = m.blksize();
        let blocks: u64 = m.blocks();

        let fdev: f64 = ulong2double(dev)?;
        let fino: f64 = ulong2double(ino)?;
        let fnlink: f64 = ulong2double(nlink)?;
        let frdev: f64 = ulong2double(rdev)?;
        let fsize: f64 = ulong2double(size)?;
        let fblksize: f64 = ulong2double(blksize)?;
        let fblocks: f64 = ulong2double(blocks)?;

        let fatime: f64 = long2double(atime)?;
        let fmtime: f64 = long2double(mtime)?;
        let fctime: f64 = long2double(ctime)?;

        let fmode: f64 = mode.into();
        let fuid: f64 = uid.into();
        let fgid: f64 = gid.into();

        self.upsert_double("dev", fdev);
        self.upsert_double("ino", fino);
        self.upsert_double("nlink", fnlink);
        self.upsert_double("rdev", frdev);
        self.upsert_double("size", fsize);
        self.upsert_double("blksize", fblksize);
        self.upsert_double("blocks", fblocks);
        self.upsert_double("atime", fatime);
        self.upsert_double("mtime", fmtime);
        self.upsert_double("ctime", fctime);
        self.upsert_double("mode", fmode);
        self.upsert_double("uid", fuid);
        self.upsert_double("gid", fgid);

        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl DuInfoDTO {
    pub fn import_metadata4mac<M>(&mut self, m: &M) -> Result<(), io::Error>
    where
        M: unix::fs::MetadataExt + darwin::fs::MetadataExt,
    {
        // user defined flags for file
        let uflags: u32 = m.st_flags();

        // file generation number
        let ugen: u32 = m.st_gen();

        let fflags: f64 = uflags.into();
        let fgen: f64 = ugen.into();

        self.upsert_double("flags", fflags);
        self.upsert_double("gen", fgen);

        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl DuInfoDTO {
    pub fn import_metadata4both(&mut self, m: &Metadata) -> Result<(), io::Error> {
        self.import_metadata4unix(m)?;
        self.import_metadata4mac(m)
    }
}

#[cfg(target_os = "macos")]
impl DuInfoDTO {
    pub fn import_metadata(&mut self, m: &Metadata) -> Result<(), io::Error> {
        self.import_metadata4both(m)
    }
}

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
impl DuInfoDTO {
    pub fn import_metadata(&mut self, m: &Metadata) -> Result<(), io::Error> {
        self.import_metadata4unix(m)
    }
}

pub trait DuInfoSink {
    fn consume(&mut self, dto: &DuInfoDTO) -> Result<(), io::Error>;

    #[cfg(target_family = "unix")]
    fn consume_meta(
        &mut self,
        buf: &mut DuInfoDTO,
        m: &Metadata,
        filename: String,
    ) -> Result<(), io::Error> {
        buf.clear();
        buf.import_metadata(m)?;
        buf.import_filename(filename);
        self.consume(buf)
    }

    #[cfg(not(target_family = "unix"))]
    fn consume_meta(
        &mut self,
        _buf: &mut DuInfoDTO,
        _m: &Metadata,
        _filename: String,
    ) -> Result<(), io::Error> {
        Err(io::Error::other("unsupported platform"))
    }

    fn consume_filename(&mut self, buf: &mut DuInfoDTO, name: String) -> Result<(), io::Error> {
        let rmet: Result<Metadata, _> = std::fs::metadata(&name);
        match rmet {
            Ok(m) => self.consume_meta(buf, &m, name),
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => {
                    buf.import_not_found(name, e.raw_os_error());
                    self.consume(buf)
                }
                _ => {
                    buf.import_other_err(name, e.raw_os_error());
                    self.consume(buf)
                }
            },
        }
    }

    fn consume_filenames<I>(&mut self, names: I) -> Result<(), io::Error>
    where
        I: Iterator<Item = Result<String, io::Error>>,
    {
        let mut buf: DuInfoDTO = DuInfoDTO::default();
        for rname in names {
            let name: String = rname?;
            self.consume_filename(&mut buf, name)?;
        }
        Ok(())
    }

    fn consume_bufread<R>(&mut self, rdr: R) -> Result<(), io::Error>
    where
        R: BufRead,
    {
        self.consume_filenames(rdr.lines())
    }
}

impl<F> DuInfoSink for F
where
    F: FnMut(&DuInfoDTO) -> Result<(), io::Error>,
{
    fn consume(&mut self, dto: &DuInfoDTO) -> Result<(), io::Error> {
        self(dto)
    }
}

pub fn writer2sink<W>(mut wtr: W) -> impl DuInfoSink
where
    W: Write,
{
    move |dto: &DuInfoDTO| {
        let s: &Struct = &dto.0;
        let dw: &mut dyn Write = &mut wtr;
        s.write_length_delimited_to_writer(dw)
            .map_err(io::Error::other)
    }
}

pub fn rdr2names2dtos2protos2wtr<R, W>(rdr: R, mut wtr: W) -> Result<(), io::Error>
where
    R: BufRead,
    W: Write,
{
    let mut sink = writer2sink(&mut wtr); // DuInfoSink
    sink.consume_bufread(rdr)?;
    drop(sink);
    wtr.flush()
}

pub fn stdin2names2dtos2protos2stdout() -> Result<(), io::Error> {
    let o = io::stdout();
    let mut ol = o.lock();
    rdr2names2dtos2protos2wtr(io::stdin().lock(), BufWriter::new(&mut ol))?;
    ol.flush()
}
