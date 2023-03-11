package io.matt.struct.facade;

public class Facade {
    private Memory memory;
    private CPU cpu;
    private HardDisk hardDisk;
    private OS os;

    public Facade() {
        memory = new Memory();
        cpu = new CPU();
        hardDisk = new HardDisk();
        os = new OS();
    }

    void powerOn() {
        System.out.println("power ON.....");
        memory.selfCheck();
        cpu.run();
        hardDisk.read();
        os.load();
        System.out.println("ready! Start");
    }
}

// subsystem: Memory
class Memory {

    public void selfCheck() {
        System.out.println("memory self checking .......");
    }
}

// subsystem: CPU
class CPU {

    public void run() {
        System.out.println("running cpu .......");
    }
}

class HardDisk {

    public void read() {
        System.out.println("reading hardDisk .........");
    }
}

class OS {
    void load() {
        System.out.println("loading os ...........");
    }
}


