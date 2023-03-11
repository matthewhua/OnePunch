package io.matt.struct.decorator;

import java.io.File;
import java.io.FileOutputStream;
import java.io.FileReader;
import java.io.IOException;

public class BaseFileDataLoader implements DataLoader {

    private String filePath;

    public BaseFileDataLoader(String filePath) {
        this.filePath = filePath;
    }

    @Override
    public String read() {
        char[] buffer = null;
        File file = new File(filePath);
        try (FileReader fileReader = new FileReader(file)) {
            buffer = new char[(int) file.length()];
            fileReader.read(buffer);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        return new String(buffer);
    }

    @Override
    public void write(String data) {
        File file = new File(filePath);
        try (FileOutputStream fos = new FileOutputStream(file)){
            fos.write(data.getBytes(), 0, data.length());
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}
