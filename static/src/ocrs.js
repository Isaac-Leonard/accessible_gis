let wasm;

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_0.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getUint32Memory0();
    const slice = mem.subarray(ptr / 4, ptr / 4 + len);
    const result = [];
    for (let i = 0; i < slice.length; i++) {
        result.push(wasm.__wbindgen_export_0.get(slice[i]));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_0.set(idx, obj);
    return idx;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getUint32Memory0();
    for (let i = 0; i < array.length; i++) {
        mem[ptr / 4 + i] = addToExternrefTable0(array[i]);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedFloat32Memory0 = null;

function getFloat32Memory0() {
    if (cachedFloat32Memory0 === null || cachedFloat32Memory0.byteLength === 0) {
        cachedFloat32Memory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32Memory0;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

const DetectedLineFinalization = new FinalizationRegistry(ptr => wasm.__wbg_detectedline_free(ptr >>> 0));
/**
* A line of text that has been detected, but not recognized.
*
* This contains information about the location of the text, but not the
* string contents.
*/
export class DetectedLine {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(DetectedLine.prototype);
        obj.__wbg_ptr = ptr;
        DetectedLineFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    static __unwrap(jsValue) {
        if (!(jsValue instanceof DetectedLine)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DetectedLineFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_detectedline_free(ptr);
    }
    /**
    * @returns {RotatedRect}
    */
    rotatedRect() {
        const ret = wasm.detectedline_rotatedRect(this.__wbg_ptr);
        return RotatedRect.__wrap(ret);
    }
    /**
    * @returns {(RotatedRect)[]}
    */
    words() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.detectedline_words(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const ImageFinalization = new FinalizationRegistry(ptr => wasm.__wbg_image_free(ptr >>> 0));
/**
* A pre-processed image that can be passed as input to `OcrEngine.loadImage`.
*/
export class Image {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Image.prototype);
        obj.__wbg_ptr = ptr;
        ImageFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_image_free(ptr);
    }
    /**
    * Return the number of channels in the image.
    * @returns {number}
    */
    channels() {
        const ret = wasm.image_channels(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
    * Return the width of the image.
    * @returns {number}
    */
    width() {
        const ret = wasm.image_width(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
    * Return the height of the image.
    * @returns {number}
    */
    height() {
        const ret = wasm.image_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
    * Return the image data in row-major, channels-last order.
    * @returns {Uint8Array}
    */
    data() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.image_data(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const OcrEngineFinalization = new FinalizationRegistry(ptr => wasm.__wbg_ocrengine_free(ptr >>> 0));
/**
* OcrEngine is the main API for performing OCR in WebAssembly.
*/
export class OcrEngine {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OcrEngineFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ocrengine_free(ptr);
    }
    /**
    * Construct a new `OcrEngine` using the models and other settings given
    * by `init`.
    *
    * To detect text in an image, `init` must have a detection model set.
    * To recognize text, `init` must have a recognition model set.
    * @param {OcrEngineInit} init
    */
    constructor(init) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(init, OcrEngineInit);
            var ptr0 = init.__destroy_into_raw();
            wasm.ocrengine_new(retptr, ptr0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeFromExternrefTable0(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Prepare an image for analysis by the OCR engine.
    *
    * The image is an array of pixels in row-major, channels last order. This
    * matches the format of the
    * [ImageData](https://developer.mozilla.org/en-US/docs/Web/API/ImageData)
    * API. Supported channel combinations are RGB and RGBA. The number of
    * channels is inferred from the length of `data`.
    * @param {number} width
    * @param {number} height
    * @param {Uint8Array} data
    * @returns {Image}
    */
    loadImage(width, height, data) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.ocrengine_loadImage(retptr, this.__wbg_ptr, width, height, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeFromExternrefTable0(r1);
            }
            return Image.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Detect text in an image.
    *
    * Returns a list of lines that were found. These can be passed to
    * `recognizeText` identify the characters.
    * @param {Image} image
    * @returns {(DetectedLine)[]}
    */
    detectText(image) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(image, Image);
            wasm.ocrengine_detectText(retptr, this.__wbg_ptr, image.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            var r3 = getInt32Memory0()[retptr / 4 + 3];
            if (r3) {
                throw takeFromExternrefTable0(r2);
            }
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Recognize text that was previously detected with `detectText`.
    *
    * Returns a list of `TextLine` objects that can be used to query the text
    * and bounding boxes of each line.
    * @param {Image} image
    * @param {(DetectedLine)[]} lines
    * @returns {(TextLine)[]}
    */
    recognizeText(image, lines) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(image, Image);
            const ptr0 = passArrayJsValueToWasm0(lines, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.ocrengine_recognizeText(retptr, this.__wbg_ptr, image.__wbg_ptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            var r3 = getInt32Memory0()[retptr / 4 + 3];
            if (r3) {
                throw takeFromExternrefTable0(r2);
            }
            var v2 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v2;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Detect and recognize text in an image.
    *
    * Returns a single string containing all the text found in reading order.
    * @param {Image} image
    * @returns {string}
    */
    getText(image) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(image, Image);
            wasm.ocrengine_getText(retptr, this.__wbg_ptr, image.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            var r3 = getInt32Memory0()[retptr / 4 + 3];
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
    * Detect and recognize text in an image.
    *
    * Returns a list of `TextLine` objects that can be used to query the text
    * and bounding boxes of each line.
    * @param {Image} image
    * @returns {(TextLine)[]}
    */
    getTextLines(image) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(image, Image);
            wasm.ocrengine_getTextLines(retptr, this.__wbg_ptr, image.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            var r3 = getInt32Memory0()[retptr / 4 + 3];
            if (r3) {
                throw takeFromExternrefTable0(r2);
            }
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const OcrEngineInitFinalization = new FinalizationRegistry(ptr => wasm.__wbg_ocrengineinit_free(ptr >>> 0));
/**
* Options for constructing an [OcrEngine].
*/
export class OcrEngineInit {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OcrEngineInitFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ocrengineinit_free(ptr);
    }
    /**
    */
    constructor() {
        const ret = wasm.ocrengineinit_new();
        this.__wbg_ptr = ret >>> 0;
        return this;
    }
    /**
    * Load a model for text detection.
    * @param {Uint8Array} data
    */
    setDetectionModel(data) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.ocrengineinit_setDetectionModel(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            if (r1) {
                throw takeFromExternrefTable0(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Load a model for text recognition.
    * @param {Uint8Array} data
    */
    setRecognitionModel(data) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.ocrengineinit_setRecognitionModel(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            if (r1) {
                throw takeFromExternrefTable0(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const RotatedRectFinalization = new FinalizationRegistry(ptr => wasm.__wbg_rotatedrect_free(ptr >>> 0));
/**
*/
export class RotatedRect {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(RotatedRect.prototype);
        obj.__wbg_ptr = ptr;
        RotatedRectFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        RotatedRectFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_rotatedrect_free(ptr);
    }
    /**
    * Return an array of the X and Y coordinates of corners of this rectangle,
    * arranged as `[x0, y0, ... x3, y3]`.
    * @returns {Float32Array}
    */
    corners() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.rotatedrect_corners(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayF32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Return the coordinates of the axis-aligned bounding rectangle of this
    * rotated rect.
    *
    * The result is a `[left, top, right, bottom]` array of coordinates.
    * @returns {Float32Array}
    */
    boundingRect() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.rotatedrect_boundingRect(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayF32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const TextLineFinalization = new FinalizationRegistry(ptr => wasm.__wbg_textline_free(ptr >>> 0));
/**
* A sequence of `TextWord`s that were recognized, forming a line.
*/
export class TextLine {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TextLine.prototype);
        obj.__wbg_ptr = ptr;
        TextLineFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TextLineFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_textline_free(ptr);
    }
    /**
    * @returns {string}
    */
    text() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.textline_text(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {(TextWord)[]}
    */
    words() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.textline_words(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const TextWordFinalization = new FinalizationRegistry(ptr => wasm.__wbg_textword_free(ptr >>> 0));
/**
* Bounding box and text of a word that was recognized.
*/
export class TextWord {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TextWord.prototype);
        obj.__wbg_ptr = ptr;
        TextWordFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TextWordFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_textword_free(ptr);
    }
    /**
    * @returns {string}
    */
    text() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.textword_text(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * Return the oriented bounding rectangle containing the characters in
    * this word.
    * @returns {RotatedRect}
    */
    rotatedRect() {
        const ret = wasm.textword_rotatedRect(this.__wbg_ptr);
        return RotatedRect.__wrap(ret);
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_rotatedrect_new = function(arg0) {
        const ret = RotatedRect.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_textline_new = function(arg0) {
        const ret = TextLine.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_textword_new = function(arg0) {
        const ret = TextWord.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_detectedline_new = function(arg0) {
        const ret = DetectedLine.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_detectedline_unwrap = function(arg0) {
        const ret = DetectedLine.__unwrap(arg0);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_0;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };

    return imports;
}

function __wbg_init_memory(imports, maybe_memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedFloat32Memory0 = null;
    cachedInt32Memory0 = null;
    cachedUint32Memory0 = null;
    cachedUint8Memory0 = null;

    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(input) {
    if (wasm !== undefined) return wasm;

    if (typeof input === 'undefined') {
        input = new URL('ocrs_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await input, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync }
export default __wbg_init;
