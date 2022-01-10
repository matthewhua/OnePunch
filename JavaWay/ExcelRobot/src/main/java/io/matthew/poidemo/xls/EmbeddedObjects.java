package io.matthew.poidemo.xls;

import org.apache.poi.hssf.usermodel.HSSFObjectData;
import org.apache.poi.hssf.usermodel.HSSFWorkbook;
import org.apache.poi.poifs.filesystem.DirectoryNode;
import org.apache.poi.poifs.filesystem.POIFSFileSystem;

import java.io.Closeable;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.IOException;

public final class EmbeddedObjects {
    private EmbeddedObjects() {}

    public static void main(String[] args) throws Exception {
/*        try (FileInputStream fis = new FileInputStream(args[0]);
             POIFSFileSystem fs = new POIFSFileSystem(fis);
             HSSFWorkbook workbook = new HSSFWorkbook(fs)){

            for (HSSFObjectData obj : workbook.getAllEmbeddedObjects()) {
                //the OLE2 Class Name of the object

                String oleName = obj.getOLE2ClassName();
                DirectoryNode dn = (obj.hasDirectoryEntry()) ? (DirectoryNode) obj.getDirectory() : null;
                Closeable document = null;
                switch (oleName) {
                    case "Worksheet":
                        document = new HSSFWorkbook(dn, fs, false);
                        break;
                    *//*case "Document":
                        document = new HWPFDocument(dn);
                        break;
                    case "Presentation":
                        document = new HSLFSlideShow(dn);
                        break;*//*
            }
        }
    }*/
    }
}
