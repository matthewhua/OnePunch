package io.matthew.poidemo.util;

import org.apache.poi.util.TempFile;

import java.io.File;

public class TempFileUtils {
    private TempFileUtils() {
    }

    @SuppressWarnings("java:S106")
    public static void checkTempFiles() {
        String tmpDir = System.getProperty(TempFile.JAVA_IO_TMPDIR) + "/poifiles";
        final File tempDir = new File(tmpDir);

    }

}
