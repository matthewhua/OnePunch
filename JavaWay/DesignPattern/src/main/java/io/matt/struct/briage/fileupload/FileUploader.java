package io.matt.struct.briage.fileupload;

public interface FileUploader {
    Object upload(String path);

    boolean check(Object object);
}
