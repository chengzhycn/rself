#[allow(dead_code)]
pub mod elf {
    use core::fmt::{self, Display};
    use std::fs::{self, File};
    use std::io::{self, prelude::*};

    pub struct Options {
        pub file_header: bool,
        pub program_headers: bool,
        pub section_headers: bool,
    }

    pub struct Elf {
        header: Option<Elf64Ehdr>,
        program_headers: Option<Vec<Elf64Phdr>>,
        section_headers: Option<Vec<Elf64Shdr>>,
    }

    impl Elf {
        pub fn new(path: &str, options: Options) -> Elf {
            let mut f = File::open(path).unwrap();
            let ehdr = Elf64Ehdr::from_file(&mut f).unwrap();

            let mut header = None;
            let mut program_headers = None;
            let mut section_headers = None;

            if options.file_header {
                header = Some(ehdr);
            }

            if options.program_headers {
                let mut phdrs: Vec<Elf64Phdr> = Vec::new();
                let mut poff = ehdr.e_phoff;

                for _i in 0..ehdr.e_phnum {
                    phdrs.push(Elf64Phdr::from_file(&mut f, poff).unwrap());
                    poff = poff + ehdr.e_phentsize as u64;

                    // TODO: why can't this work?
                    // let poff = poff + ehdr.e_phentsize as u64;
                }

                program_headers = Some(phdrs);
            }

            if options.section_headers {
                let mut shdrs: Vec<Elf64Shdr> = Vec::new();
                let mut soff = ehdr.e_shoff;

                for _i in 0..ehdr.e_shnum {
                    shdrs.push(Elf64Shdr::from_file(&mut f, soff).unwrap());
                    soff = soff + ehdr.e_shensize as u64;
                }

                section_headers = Some(shdrs);
            }

            Elf {
                header,
                program_headers,
                section_headers,
            }
        }

        pub fn to_str(&mut self, buf: &mut dyn Write) -> io::Result<()> {
            // Elf64Ehdr implements Copy trait, so no ownership moved here.
            if let Some(ehdr) = self.header {
                buf.write_fmt(format_args!("ELF Header:\n"))?;
                buf.write_fmt(format_args!("{}", ehdr))?;
            }

            // Vec<> doesn't implement Copy trait, so if we don't add as_deref,
            // data move happens.
            if let Some(phdrs) = self.program_headers.as_deref() {
                buf.write_fmt(format_args!("Program Headers:\n"))?;
                buf.write_fmt(format_args!("{}", Elf::phdr_header()))?;

                for phdr in phdrs {
                    buf.write_fmt(format_args!("{}", &phdr))?;
                }

                buf.write_fmt(format_args!("{}", Elf::phdr_footer()))?;
            }

            if let Some(shdrs) = self.section_headers.as_deref() {
                buf.write_fmt(format_args!("Section Headers:\n"))?;
                buf.write_fmt(format_args!("{}", Elf::shdr_header()))?;

                for (i, shdr) in shdrs.iter().enumerate() {
                    unsafe { SH_INDEX = i as u16 };
                    buf.write_fmt(format_args!("{}", shdr))?;
                }

                buf.write_fmt(format_args!("{}", Elf::shdr_footer()))?;
            }

            buf.write_fmt(format_args!("\n"))
        }

        fn phdr_header() -> String {
            format!(
                "  {:<16}{:<18} {:<18} {:<18}\n  {:<16}{:<18} {:<18}  {:<6} {}\n",
                "Type", "Offset", "VirtAddr", "PhysAddr", "", "FileSiz", "MemSiz", "Flags", "Align"
            )
        }

        fn phdr_footer() -> String {
            format!("\n")
        }

        fn shdr_header() -> String {
            format!(
                "  [{:<2}] {:<16}  {:<16} {:<16}  {:<8}\n  {:<4} {:<17} {:<16} {:<6} {:<5} {:<5} {:<7}\n",
                "Nr", "Name", "Type", "Address", "Offset", "", "Size", "EntSize", "Flags", "Link", "Info", "Align"
            )
        }

        fn shdr_footer() -> String {
            String::from(
                "Key to Flags:
  W (write), A (alloc), X (execute), M (merge), S (strings), I (info),
  L (link order), O (extra OS processing required), G (group), T (TLS),
  C (compressed), x (unknown), o (OS specific), E (exclude),
  l (large), p (processor specific)",
            )
        }
    }

    // TODO: how can we implement fmt::Display trait here?
    // impl<'a> fmt::Display for Elf<'a> {
    //     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //         write!(f, "{}", Elf64Ehdr::from_file(self.f).unwrap())
    //     }
    // }

    const EI_NIDENT: usize = 16;

    const ELFOSABI_NONE: &str = "UNIX System V ABI"; /* 0x0 */
    const ELFOSABI_SYSV: &str = ELFOSABI_NONE; /* 0x0 */
    const ELFOSABI_HPUX: &str = "HP-UX"; /* 0x1 */
    const ELFOSABI_NETBSD: &str = "NetBSD"; /* 0x2 */
    const ELFOSABI_GNU: &str = "Object uses GNU ELF extensions"; /* 0x3 */
    const ELFOSABI_LINUX: &str = ELFOSABI_GNU; /* 0x3 */
    const ELFOSABI_SOLARIS: &str = "Sun Solaris"; /* 0x6 */
    const ELFOSABI_AIX: &str = "IBM AIX"; /* 0x7 */
    const ELFOSABI_IRIX: &str = "SGI Irix"; /* 0x8 */
    const ELFOSABI_FREEBSD: &str = "FreeBSD"; /* 0x9 */
    const ELFOSABI_TRU64: &str = "Compaq TRU64 UNIX"; /* 0xA */
    const ELFOSABI_MODESTO: &str = "Novell Modesto"; /* 0xB */
    const ELFOSABI_OPENBSD: &str = "OpenBSD"; /* 0xC */
    const ELFOSABI_ARM_AEABI: &str = "ARM EABI"; /* 0x40 */
    const ELFOSABI_ARM: &str = "ARM"; /* 0x61 */
    const ELFOSABI_STANDALONE: &str = "Standalone (embedded) application"; /* 0xFF */

    const ET_NONE: &str = "No file type"; /* 0x0 */
    const ET_REL: &str = "Relocatable file"; /* 0x1 */
    const ET_EXEC: &str = "Executable file"; /* 0x2 */
    const ET_DYN: &str = "Shared object file"; /* 0x3 */
    const ET_CORE: &str = "Core file"; /* 0x4 */
    const ET_LOOS: &str = "OS-specific range start"; /* 0xfe00 */
    const ET_HIOS: &str = "OS-specific range end"; /* 0xfeff */
    const ET_LOPROC: &str = "Processor-specific range start"; /* 0xff00 */
    const ET_HIPROC: &str = "Processor-specific range end"; /* 0xffff */

    const EM_ARRAY: [&str; 256] = [
        "No machine",
        "AT&T WE 32100",
        "SUN SPARC",
        "Intel 80386",
        "Motorola m68k family",
        "Motorola m88k family",
        "Intel MCU",
        "Intel 80860",
        "MIPS R3000 big-endian",
        "IBM System/370",
        "MIPS R3000 little-endian",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved", /* reserved 11-14 */
        "HPPA",
        "Reserved", /* reserved 16 */
        "Fujitsu VPP500",
        "Sun's v8plus",
        "Intel 80960",
        "PowerPC",
        "PowerPC 64-bit",
        "IBM S390",
        "IBM SPU/SPC",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved", /* reserved 24-35 */
        "NEC V800 series",
        "Fujitsu FR20",
        "TRW RH-32",
        "Motorola RCE",
        "ARM",
        "Digital Alpha",
        "Hitachi SH",
        "SPARC v9 64-bit",
        "Siemens Tricore",
        "Argonaut RISC Core",
        "Hitachi H8/300",
        "Hitachi H8/300H",
        "Hitachi H8S",
        "Hitachi H8/500",
        "Intel Merced",
        "Stanford MIPS-X",
        "Motorola Coldfire",
        "Motorola M68HC12",
        "Fujitsu MMA Multimedia Accelerator",
        "Siemens PCP",
        "Sony nCPU embeeded RISC",
        "Denso NDR1 microprocessor",
        "Motorola Start*Core processor",
        "Toyota ME16 processor",
        "STMicroelectronic ST100 processor",
        "Advanced Logic Corp. Tinyj emb.fam",
        "AMD x86-64 architecture",
        "Sony DSP Processor",
        "Digital PDP-10",
        "Digital PDP-11",
        "Siemens FX66 microcontroller",
        "STMicroelectronics ST9+ 8/16 mc",
        "STmicroelectronics ST7 8 bit mc",
        "Motorola MC68HC16 microcontroller",
        "Motorola MC68HC11 microcontroller",
        "Motorola MC68HC08 microcontroller",
        "Motorola MC68HC05 microcontroller",
        "Silicon Graphics SVx",
        "STMicroelectronics ST19 8 bit mc",
        "Digital VAX",
        "Axis Communications 32-bit emb.proc",
        "Infineon Technologies 32-bit emb.proc",
        "Element 14 64-bit DSP Processor",
        "LSI Logic 16-bit DSP Processor",
        "Donald Knuth's educational 64-bit proc",
        "Harvard University machine-independent object files",
        "SiTera Prism",
        "Atmel AVR 8-bit microcontroller",
        "Fujitsu FR30",
        "Mitsubishi D10V",
        "Mitsubishi D30V",
        "NEC v850",
        "Mitsubishi M32R",
        "Matsushita MN10300",
        "Matsushita MN10200",
        "picoJava",
        "OpenRISC 32-bit embedded processor",
        "ARC International ARCompact",
        "Tensilica Xtensa Architecture",
        "Alphamosaic VideoCore",
        "Thompson Multimedia General Purpose Proc",
        "National Semi. 32000",
        "Tenor Network TPC",
        "Trebia SNP 1000",
        "STMicroelectronics ST200",
        "Ubicom IP2xxx",
        "MAX processor",
        "National Semi. CompactRISC",
        "Fujitsu F2MC16",
        "Texas Instruments msp430",
        "Analog Devices Blackfin DSP",
        "Seiko Epson S1C33 family",
        "Sharp embedded microprocessor",
        "Arca RISC",
        "PKU-Unity & MPRC Peking Uni. mc series",
        "eXcess configurable cpu",
        "Icera Semi. Deep Execution Processor",
        "Altera Nios II",
        "National Semi. CompactRISC CRX",
        "Motorola XGATE",
        "Infineon C16x/XC16x",
        "Renesas M16C",
        "Microchip Technology dsPIC30F",
        "Freescale Communication Engine RISC",
        "Renesas M32C",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved", /* reserved 121-130 */
        "Altium TSK3000",
        "Freescale RS08",
        "Analog Devices SHARC family",
        "Cyan Technology eCOG2",
        "Sunplus S+core7 RISC",
        "New Japan Radio (NJR) 24-bit DSP",
        "Broadcom VideoCore III",
        "RISC for Lattice FPGA",
        "Seiko Epson C17",
        "Texas Instruments TMS320C6000 DSP",
        "Texas Instruments TMS320C2000 DSP",
        "Texas Instruments TMS320C55x DSP",
        "Texas Instruments App. Specific RISC",
        "Texas Instruments Prog. Realtime Unit",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved", /* reserved 145-159 */
        "STMicroelectronics 64bit VLIW DSP",
        "Cypress M8C",
        "Renesas R32C",
        "NXP Semi. TriMedia",
        "QUALCOMM DSP6",
        "Intel 8051 and variants",
        "STMicroelectronics STxP7x",
        "Andes Tech. compact code emb. RISC",
        "Cyan Technology eCOG1X",
        "Dallas Semi. MAXQ30 mc",
        "New Japan Radio (NJR) 16-bit DSP",
        "M2000 Reconfigurable RISC",
        "Cray NV2 vector architecture",
        "Renesas RX",
        "Imagination Tech. META",
        "MCST Elbrus",
        "Cyan Technology eCOG16",
        "National Semi. CompactRISC CR16",
        "Freescale Extended Time Processing Unit",
        "Infineon Tech. SLE9X",
        "Intel L10M",
        "Intel K10M",
        "Reserved", /* reserved 182 */
        "ARM AARCH64",
        "Reserved", /* reserved 184 */
        "Amtel 32-bit microprocessor",
        "STMicroelectronics STM8",
        "Tileta TILE64",
        "Tilera TILEPro",
        "Xilinx MicroBlaze",
        "NVIDIA CUDA",
        "Tilera TILE-Gx",
        "CloudShield",
        "KIPO-KAIST Core-A 1st gen.",
        "KIPO-KAIST Core-A 2nd gen.",
        "Synopsys ARCompact V2",
        "Open8 RISC",
        "Renesas RL78",
        "Broadcom VideoCore V",
        "Renesas 78KOR",
        "Freescale 56800EX DSC",
        "Beyond BA1",
        "Beyond BA2",
        "XMOS xCORE",
        "Microchip 8-bit PIC(r)",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved", /* reserved 205-209 */
        "KM211 KM32",
        "KM211 KMX32",
        "KM211 KMX16",
        "KM211 KMX8",
        "KM211 KVARC",
        "Paneve CDP",
        "Cognitive Smart Memory Processor",
        "Bluechip CoolEngine",
        "Nanoradio Optimized RISC",
        "CSR Kalimba",
        "Zilog Z80",
        "Controls and Data Services VISIUMcore",
        "FTDI Chip FT32",
        "Moxie processor",
        "AMD GPU",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved", /* reserved 225-242 */
        "RISC-V",
        "Reserved",
        "Reserved",
        "Reserved", /* reserved 244-246 */
        "Linux BPF -- in-kernel virtual machine",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved", /* reserved 248-251 */
        "C-SKY",
        "Reserved",
        "Reserved",
        "Reserved",
    ];

    #[repr(C, packed)] // TODO: why should add this?
    #[derive(Clone, Copy, Debug)]
    pub struct Elf64Ehdr {
        e_ident: [u8; EI_NIDENT],
        e_type: u16,
        e_machine: u16,
        e_version: u32,
        e_entry: u64,
        e_phoff: u64,
        e_shoff: u64,
        e_flags: u32,
        e_ehsize: u16,
        e_phentsize: u16,
        e_phnum: u16,
        e_shensize: u16,
        e_shnum: u16,
        e_shstrndx: u16,
    }

    impl Elf64Ehdr {
        pub fn from_file(f: &mut File) -> io::Result<Elf64Ehdr> {
            let mut buf = [0; 64];

            f.read_exact(&mut buf[..])?;

            let (_head, body, _tail) = unsafe { buf.align_to::<Elf64Ehdr>() };

            let elfhdr = body[0];

            Ok(elfhdr)
        }
    }

    impl Display for Elf64Ehdr {
        #[allow(unaligned_references)]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            // Magic number
            write!(f, "  {:<34} ", "Magic:")?;
            for n in self.e_ident.iter() {
                write!(f, "{:02x} ", n)?;
            }
            write!(f, "\n")?;

            // Class
            // TODO: need verify?
            write!(f, "  {:<34} ", "Class:")?;
            match self.e_ident[4] {
                1 => {
                    write!(
                        f,
                        "{}32\n",
                        std::str::from_utf8(&self.e_ident[1..4]).unwrap()
                    )?;
                }
                2 => {
                    write!(
                        f,
                        "{}64\n",
                        std::str::from_utf8(&self.e_ident[1..4]).unwrap()
                    )?;
                }
                _ => {}
            }

            // Data
            // TODO: using enums to optimize match arms.
            write!(f, "  {:<34} ", "Data:")?;
            match self.e_ident[5] {
                1 => {
                    write!(f, "2's complement, little endian\n")?;
                }
                2 => {
                    write!(f, "2's complement, big endian\n")?;
                }
                _ => {}
            }

            // Version
            write!(f, "  {:<34} {}\n", "Version:", self.e_ident[6])?;

            // OS/ABI
            write!(f, "  {:<34} ", "OS/ABI:")?;
            match self.e_ident[7] {
                0 => {
                    write!(f, "{}\n", ELFOSABI_SYSV)?;
                }
                1 => {
                    write!(f, "{}\n", ELFOSABI_HPUX)?;
                }
                2 => {
                    write!(f, "{}\n", ELFOSABI_NETBSD)?;
                }
                3 => {
                    write!(f, "{}\n", ELFOSABI_LINUX)?;
                }
                6 => {
                    write!(f, "{}\n", ELFOSABI_SOLARIS)?;
                }
                7 => {
                    write!(f, "{}\n", ELFOSABI_AIX)?;
                }
                8 => {
                    write!(f, "{}\n", ELFOSABI_IRIX)?;
                }
                9 => {
                    write!(f, "{}\n", ELFOSABI_FREEBSD)?;
                }
                10 => {
                    write!(f, "{}\n", ELFOSABI_TRU64)?;
                }
                11 => {
                    write!(f, "{}\n", ELFOSABI_MODESTO)?;
                }
                12 => {
                    write!(f, "{}\n", ELFOSABI_OPENBSD)?;
                }
                64 => {
                    write!(f, "{}\n", ELFOSABI_ARM_AEABI)?;
                }
                97 => {
                    write!(f, "{}\n", ELFOSABI_ARM)?;
                }
                255 => {
                    write!(f, "{}\n", ELFOSABI_STANDALONE)?;
                }
                _ => {}
            }

            // ABI Version
            write!(f, "  {:<34} {}\n", "ABI Version:", self.e_ident[8])?;

            // Type
            write!(f, "  {:<34} ", "Type:")?;
            match self.e_type {
                0 => {
                    write!(f, "{}\n", ET_NONE)?;
                }
                1 => {
                    write!(f, "{}\n", ET_REL)?;
                }
                2 => {
                    write!(f, "{}\n", ET_EXEC)?;
                }
                3 => {
                    write!(f, "{}\n", ET_DYN)?;
                }
                4 => {
                    write!(f, "{}\n", ET_CORE)?;
                }
                0xfe00 => {
                    write!(f, "{}\n", ET_LOOS)?;
                }
                0xfeff => {
                    write!(f, "{}\n", ET_HIOS)?;
                }
                0xff00 => {
                    write!(f, "{}\n", ET_LOPROC)?;
                }
                0xffff => {
                    write!(f, "{}\n", ET_HIPROC)?;
                }
                _ => {}
            }

            // Machine
            write!(
                f,
                "  {:<34} {}\n",
                "Machine:", EM_ARRAY[self.e_ident[8] as usize]
            )?;

            // Entry point address
            write!(f, "  {:<34} {:#x}\n", "Entry point address:", self.e_entry)?;

            // Start of program headers
            write!(
                f,
                "  {:<34} {} (bytes into file)\n",
                "Start of program headers:", self.e_phoff
            )?;

            // Start of section headers
            write!(
                f,
                "  {:<34} {} (bytes into file)\n",
                "Start of section headers:", self.e_shoff
            )?;

            // Flags
            write!(f, "  {:<34} {:#x}\n", "Flags:", self.e_flags)?;

            // Size of this header
            write!(
                f,
                "  {:<34} {} (bytes)\n",
                "Size of this header:", self.e_ehsize
            )?;

            // Size of program headers
            write!(
                f,
                "  {:<34} {} (bytes)\n",
                "Size of program headers:", self.e_phentsize
            )?;

            // Number of program headers
            write!(
                f,
                "  {:<34} {}\n",
                "Number of program headers:", self.e_phnum
            )?;

            // Size of section headers
            write!(
                f,
                "  {:<34} {} (bytes)\n",
                "Size of section headers:", self.e_shensize
            )?;

            // Number of section headers
            write!(
                f,
                "  {:<34} {}\n",
                "Number of section headers:", self.e_shnum
            )?;

            // Section header string table index
            write!(
                f,
                "  {:<34} {}\n",
                "Section header string table index:", self.e_shstrndx
            )?;

            write!(f, "\n")
        }
    }

    const PT_NULL: &str = "NULL"; /* 0x0 Program header table entry unused */
    const PT_LOAD: &str = "LOAD"; /* 0x1 Loadable program segment */
    const PT_DYNAMIC: &str = "DYNAMIC"; /* 0x2 Dynamic linking information */
    const PT_INTERP: &str = "INTERP"; /* 0x3 Program interpreter */
    const PT_NOTE: &str = "NOTE"; /* 0x4 Auxiliary information */
    const PT_SHLIB: &str = "SHLIB"; /* 0x5 Reserved */
    const PT_PHDR: &str = "PHDR"; /* 0x6 Entry for header table itself */
    const PT_TLS: &str = "TLS"; /* 0x7 Thread-local storage segment */
    const PT_NUM: &str = "NUM"; /* 0x8 Number of defined types */
    const PT_LOOS: &str = "LOOS"; /* 0x60000000 Start of OS-specific */
    const PT_GNU_EH_FRAME: &str = "GNU_EH_FRAME"; /* 0x6474e550  GCC .eh_frame_hdr segment */
    const PT_GNU_STACK: &str = "GNU_STACK"; /* 0x6474e551 Indicates stack executability */
    const PT_GNU_RELRO: &str = "GNU_RELRO"; /* 0x6474e552 Read-only after relocation */
    const PT_GNU_PROPERTY: &str = "GNU_PROPERTY"; /* 0x6474e553 cover .note.gnu.property section */
    const PT_LOSUNW: &str = "LOSUMW"; /* 0x6ffffffa */
    const PT_SUNWBSS: &str = "SUMWBSS"; /* 0x6ffffffa Sun Specific segment */
    const PT_SUNWSTACK: &str = "SUMWSTACK"; /* 0x6ffffffb Stack segment */
    const PT_HISUNW: &str = "HISUNW"; /* 0x6fffffff */
    const PT_HIOS: &str = "HIOS"; /* 0x6fffffff End of OS-specific */
    const PT_LOPROC: &str = "LOPROC"; /* 0x70000000 Start of processor-specific */
    const PT_HIPROC: &str = "HIPROC"; /* 0x7fffffff End of processor-specific */

    const PF_X: u8 = 1 << 0; /* Segment is executable */
    const PF_W: u8 = 1 << 1; /* Segment is writable */
    const PF_R: u8 = 1 << 2; /* Segment is readable */

    #[derive(Debug, Clone, Copy)]
    #[repr(C, packed)]
    struct Elf64Phdr {
        p_type: u32,
        p_flags: u32,
        p_offset: u64,
        p_vaddr: u64,
        p_paddr: u64,
        p_filesz: u64,
        p_memsz: u64,
        p_align: u64,
    }

    impl Elf64Phdr {
        fn from_file(f: &mut fs::File, off: u64) -> io::Result<Elf64Phdr> {
            let mut buf = [0; 56];

            f.seek(io::SeekFrom::Start(off))?;
            f.read_exact(&mut buf)?;

            let (_head, body, _tail) = unsafe { buf.align_to::<Elf64Phdr>() };

            let elf64_phdr = body[0];

            Ok(elf64_phdr)
        }
    }

    impl Display for Elf64Phdr {
        #[allow(unaligned_references)]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.p_type {
                0 => {
                    write!(f, "  {:<16}", PT_NULL)?;
                }
                1 => {
                    write!(f, "  {:<16}", PT_LOAD)?;
                }
                2 => {
                    write!(f, "  {:<16}", PT_DYNAMIC)?;
                }
                3 => {
                    write!(f, "  {:<16}", PT_INTERP)?;
                }
                4 => {
                    write!(f, "  {:<16}", PT_NOTE)?;
                }
                5 => {
                    write!(f, "  {:<16}", PT_SHLIB)?;
                }
                6 => {
                    write!(f, "  {:<16}", PT_PHDR)?;
                }
                7 => {
                    write!(f, "  {:<16}", PT_TLS)?;
                }
                8 => {
                    write!(f, "  {:<16}", PT_NUM)?;
                }
                0x6474e550 => {
                    write!(f, "  {:<16}", PT_GNU_EH_FRAME)?;
                }
                0x6474e551 => {
                    write!(f, "  {:<16}", PT_GNU_STACK)?;
                }
                0x6474e552 => {
                    write!(f, "  {:<16}", PT_GNU_RELRO)?;
                }
                0x6474e553 => {
                    write!(f, "  {:<16}", PT_GNU_PROPERTY)?;
                }
                0x6ffffffa => {
                    write!(f, "  {:<16}", PT_SUNWBSS)?;
                }
                0x6ffffffb => {
                    write!(f, "  {:<16}", PT_SUNWSTACK)?;
                }
                _ => {
                    write!(f, "  {:#015x}", self.p_type)?;
                }
            }

            write!(
                f,
                "{:#018x} {:#018x} {:#018x}\n",
                self.p_offset, self.p_vaddr, self.p_paddr
            )?;

            let mut flag_str = String::new();
            if ((self.p_flags >> 2) & 0x1) == 1 {
                flag_str.push('R');
            } else {
                flag_str.push(' ');
            }
            if ((self.p_flags >> 1) & 0x1) == 1 {
                flag_str.push('W');
            } else {
                flag_str.push(' ');
            }
            if (self.p_flags & 0x1) == 1 {
                flag_str.push('E');
            } else {
                flag_str.push(' ');
            }

            write!(
                f,
                "  {:<16}{:#018x} {:#018x}  {:<6} {:#x}\n",
                "", self.p_filesz, self.p_memsz, flag_str, self.p_align
            )
        }
    }

    /* Legal values for sh_type (section type).  */

    const SHT_NULL: &str = "NULL"; /* 0 Section header table entry unused */
    const SHT_PROGBITS: &str = "PROGBITS"; /* 1 Program data */
    const SHT_SYMTAB: &str = "SYMTAB"; /* 2 Symbol table */
    const SHT_STRTAB: &str = "STRTAB"; /* 3 String table */
    const SHT_RELA: &str = "RELA"; /* 4 Relocation entries with addends */
    const SHT_HASH: &str = "HASH"; /* 5 Symbol hash table */
    const SHT_DYNAMIC: &str = "DYNAMIC"; /* 6 Dynamic linking information */
    const SHT_NOTE: &str = "NOTE"; /* 7 Notes */
    const SHT_NOBITS: &str = "NOBITS"; /* 8 Program space with no data (bss) */
    const SHT_REL: &str = "REL"; /* 9 Relocation entries, no addends */
    const SHT_SHLIB: &str = "SHLIB"; /* 10 Reserved */
    const SHT_DYNSYM: &str = "DYNSYM"; /* 11 Dynamic linker symbol table */
    const SHT_INIT_ARRAY: &str = "INIT_ARRAY"; /* 14 Array of constructors */
    const SHT_FINI_ARRAY: &str = "FIMI_ARRAY"; /* 15 Array of destructors */
    const SHT_PREINIT_ARRAY: &str = "PREINIT_ARRAY"; /* 16 Array of pre-constructors */
    const SHT_GROUP: &str = "GROUP"; /* 17 Section group */
    const SHT_SYMTAB_SHNDX: &str = "SYMTAB_SHNDX"; /* 18 Extended section indeces */
    const SHT_NUM: &str = "NUM"; /* 19 Number of defined types.  */
    const SHT_LOOS: &str = "LOOS"; /* 0x60000000 Start OS-specific.  */
    const SHT_GNU_ATTRIBUTES: &str = "GNU_ATTRIBUTES"; /* 0x6ffffff5 Object attributes.  */
    const SHT_GNU_HASH: &str = "GNU_HASH"; /* 0x6ffffff6 GNU-style hash table.  */
    const SHT_GNU_LIBLIST: &str = "GNU_LIBLIST"; /* 0x6ffffff7 Prelink library list */
    const SHT_CHECKSUM: &str = "CHECKSUM"; /* 0x6ffffff8 Checksum for DSO content.  */
    const SHT_LOSUNW: &str = "LOSUNW"; /* 0x6ffffffa Sun-specific low bound.  */
    const SHT_SUNW_MOVE: &str = "SUNW_move"; /* 0x6ffffffa */
    const SHT_SUNW_COMDAT: &str = "SUNW_COMDAT"; /* 0x6ffffffb */
    const SHT_SUNW_SYMINFO: &str = "SUNW_syminfo"; /* 0x6ffffffc */
    const SHT_GNU_VERDEF: &str = "GNU_verdef"; /* 0x6ffffffd Version definition section.  */
    const SHT_GNU_VERNEED: &str = "GNU_verneed"; /* 0x6ffffffe Version needs section.  */
    const SHT_GNU_VERSYM: &str = "GNU_versym"; /* 0x6fffffff Version symbol table.  */
    const SHT_HISUNW: &str = "HISUNW"; /* 0x6fffffff Sun-specific high bound.  */
    const SHT_HIOS: &str = "HIOS"; /* 0x6fffffff End OS-specific type */
    const SHT_LOPROC: &str = "LOPROC"; /* 0x70000000 Start of processor-specific */
    const SHT_HIPROC: &str = "HIPROC"; /* 0x7fffffff End of processor-specific */
    const SHT_LOUSER: &str = "LOUSER"; /* 0x80000000 Start of application-specific */
    const SHT_HIUSER: &str = "HIUSER"; /* 0x8fffffff End of application-specific */

    /* Legal values for sh_flags (section flags).  */

    const SHF_WRITE: u32 = 1 << 0; /* Writable */
    const SHF_ALLOC: u32 = 1 << 1; /* Occupies memory during execution */
    const SHF_EXECINSTR: u32 = 1 << 2; /* Executable */
    const SHF_MERGE: u32 = 1 << 4; /* Might be merged */
    const SHF_STRINGS: u32 = 1 << 5; /* Contains nul-terminated strings */
    const SHF_INFO_LINK: u32 = 1 << 6; /* `sh_info' contains SHT index */
    const SHF_LINK_ORDER: u32 = 1 << 7; /* Preserve order after combining */
    const SHF_OS_NONCONFORMING: u32 = 1 << 8; /* Non-standard OS specific handling required */
    const SHF_GROUP: u32 = 1 << 9; /* Section is member of a group.  */
    const SHF_TLS: u32 = 1 << 10; /* Section hold thread-local data.  */
    const SHF_COMPRESSED: u32 = 1 << 11; /* Section with compressed data. */
    const SHF_MASKOS: u32 = 0x0ff00000; /* OS-specific.  */
    const SHF_MASKPROC: u32 = 0xf0000000; /* Processor-specific */
    const SHF_ORDERED: u32 = 1 << 30; /* Special ordering requirement (Solaris).  */
    const SHF_EXCLUDE: u32 = 1 << 31; /* Section is excluded unless referenced or allocated (Solaris).*/

    #[derive(Debug, Clone, Copy)]
    #[repr(C, packed)]
    struct Elf64Shdr {
        sh_name: u32,
        sh_type: u32,
        sh_flags: u64,
        sh_addr: u64,
        sh_offset: u64,
        sh_size: u64,
        sh_link: u32,
        sh_info: u32,
        sh_addralign: u64,
        sh_entsize: u64,
    }

    static mut SH_INDEX: u16 = 0;
    static mut SH_STRTABLE: Vec<u8> = Vec::new();

    impl Elf64Shdr {
        fn from_file(f: &mut fs::File, off: u64) -> io::Result<Elf64Shdr> {
            let mut buf = [0; 64];

            f.seek(io::SeekFrom::Start(off))?;
            f.read_exact(&mut buf)?;

            let (_head, body, _tail) = unsafe { buf.align_to::<Elf64Shdr>() };

            let elf64_shdr = body[0];

            Ok(elf64_shdr)
        }
    }

    impl Display for Elf64Shdr {
        #[allow(unaligned_references)]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            unsafe {
                write!(f, "  [{:<2}] {:<17} ", SH_INDEX, self.sh_name)?;
            }

            match self.sh_type {
                0 => {
                    write!(f, "{:<17} ", SHT_NULL)?;
                }
                1 => {
                    write!(f, "{:<17} ", SHT_PROGBITS)?;
                }
                2 => {
                    write!(f, "{:<17} ", SHT_SYMTAB)?;
                }
                3 => {
                    write!(f, "{:<17} ", SHT_STRTAB)?;
                }
                4 => {
                    write!(f, "{:<17} ", SHT_RELA)?;
                }
                5 => {
                    write!(f, "{:<17} ", SHT_HASH)?;
                }
                6 => {
                    write!(f, "{:<17} ", SHT_DYNAMIC)?;
                }
                7 => {
                    write!(f, "{:<17} ", SHT_NOTE)?;
                }
                8 => {
                    write!(f, "{:<17} ", SHT_NOBITS)?;
                }
                9 => {
                    write!(f, "{:<17} ", SHT_REL)?;
                }
                10 => {
                    write!(f, "{:<17} ", SHT_SHLIB)?;
                }
                11 => {
                    write!(f, "{:<17} ", SHT_DYNSYM)?;
                }
                14 => {
                    write!(f, "{:<17} ", SHT_INIT_ARRAY)?;
                }
                15 => {
                    write!(f, "{:<17} ", SHT_FINI_ARRAY)?;
                }
                16 => {
                    write!(f, "{:<17} ", SHT_PREINIT_ARRAY)?;
                }
                17 => {
                    write!(f, "{:<17} ", SHT_GROUP)?;
                }
                18 => {
                    write!(f, "{:<17} ", SHT_SYMTAB_SHNDX)?;
                }
                19 => {
                    write!(f, "{:<17} ", SHT_NUM)?;
                }
                0x6ffffff5 => {
                    write!(f, "{:<17} ", SHT_GNU_ATTRIBUTES)?;
                }
                0x6ffffff6 => {
                    write!(f, "{:<17} ", SHT_GNU_HASH)?;
                }
                0x6ffffff7 => {
                    write!(f, "{:<17} ", SHT_GNU_LIBLIST)?;
                }
                0x6ffffff8 => {
                    write!(f, "{:<17} ", SHT_CHECKSUM)?;
                }
                0x6ffffffa => {
                    write!(f, "{:<17} ", SHT_SUNW_MOVE)?;
                }
                0x6ffffffb => {
                    write!(f, "{:<17} ", SHT_SUNW_COMDAT)?;
                }
                0x6ffffffc => {
                    write!(f, "{:<17} ", SHT_SUNW_SYMINFO)?;
                }
                0x6ffffffd => {
                    write!(f, "{:<17} ", SHT_GNU_VERDEF)?;
                }
                0x6ffffffe => {
                    write!(f, "{:<17} ", SHT_GNU_VERNEED)?;
                }
                0x6fffffff => {
                    write!(f, "{:<17} ", SHT_GNU_VERSYM)?;
                }
                _ => {
                    write!(f, "{:<17} ", "")?;
                }
            }

            write!(f, "{:016x}  {:08x}\n", self.sh_addr, self.sh_offset)?;

            let mut flag_str = String::new();
            if (self.sh_flags & 0x1) == 1 {
                flag_str.push('W');
            }
            if (self.sh_flags >> 1) & 0x1 == 1 {
                flag_str.push('A');
            }
            if (self.sh_flags >> 2) & 0x1 == 1 {
                flag_str.push('X');
            }
            if (self.sh_flags >> 4) & 0x1 == 1 {
                flag_str.push('M');
            }
            if (self.sh_flags >> 5) & 0x1 == 1 {
                flag_str.push('S');
            }
            if (self.sh_flags >> 6) & 0x1 == 1 {
                flag_str.push('I');
            }
            if (self.sh_flags >> 7) & 0x1 == 1 {
                flag_str.push('L');
            }
            if (self.sh_flags >> 8) & 0x1 == 1 {
                flag_str.push('O');
            }
            if (self.sh_flags >> 9) & 0x1 == 1 {
                flag_str.push('G');
            }
            if (self.sh_flags >> 10) & 0x1 == 1 {
                flag_str.push('T');
            }
            if (self.sh_flags >> 11) & 0x1 == 1 {
                flag_str.push('C');
            }
            if (self.sh_flags >> 31) & 0x1 == 1 {
                flag_str.push('E');
            }

            write!(
                f,
                "  {:<4} {:016x}  {:016x} {:<6} {:<5} {:<5} {:<7}\n",
                "",
                self.sh_size,
                self.sh_entsize,
                flag_str,
                self.sh_link,
                self.sh_info,
                self.sh_addralign
            )
        }
    }
}
