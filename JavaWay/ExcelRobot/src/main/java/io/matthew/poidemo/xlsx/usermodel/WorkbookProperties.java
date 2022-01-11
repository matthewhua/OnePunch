package io.matthew.poidemo.xlsx.usermodel;

import org.apache.poi.ooxml.POIXMLProperties;
import org.apache.poi.xssf.usermodel.XSSFWorkbook;

import java.io.FileOutputStream;
import java.io.IOException;

/**
 *  How to set extended and custom properties
 */
public class WorkbookProperties {
    public static void main(String[]args) throws IOException {
        try (XSSFWorkbook workbook = new XSSFWorkbook()) {
            workbook.createSheet("Workbook Properties");

            POIXMLProperties props = workbook.getProperties();

            /**
             * Extended properties are a predefined set of metadata properties
             * that are specifically applicable to Office Open XML documents.
             * Extended properties consist of 24 simple properties and 3 complex properties stored in the
             *  part targeted by the relationship of type
             */
            POIXMLProperties.ExtendedProperties ext = props.getExtendedProperties();
            ext.getUnderlyingProperties().setCompany("Apache Software Foundation");
            ext.getUnderlyingProperties().setTemplate("XSSF");

            /*
             * Custom properties enable users to define custom metadata properties.
             */

            POIXMLProperties.CustomProperties cust = props.getCustomProperties();
            cust.addProperty("Author", "John Smith");
            cust.addProperty("Year", 2009);
            cust.addProperty("Price", 45.50);
            cust.addProperty("Available", true);

            try (FileOutputStream out = new FileOutputStream("workbook.xlsx")) {
                workbook.write(out);
            }
        }
    }
}
