const fs = require("fs");
const qs = require("qs");

function normalizeDelimiter(delimiter) {
    if (!delimiter || typeof delimiter !== "object") {
        return "&";
    }

    if (delimiter.kind === "regex") {
        return new RegExp(delimiter.value);
    }

    return delimiter.value;
}

function normalizeDecodeOptions(options) {
    return {
        allowDots: !!options.allowDots,
        decodeDotInKeys: !!options.decodeDotInKeys,
        allowEmptyArrays: !!options.allowEmptyArrays,
        allowSparse: !!options.allowSparse,
        arrayLimit: options.arrayLimit,
        charset: options.charset,
        charsetSentinel: !!options.charsetSentinel,
        comma: !!options.comma,
        delimiter: normalizeDelimiter(options.delimiter),
        depth: options.depth,
        duplicates: options.duplicates,
        ignoreQueryPrefix: !!options.ignoreQueryPrefix,
        interpretNumericEntities: !!options.interpretNumericEntities,
        parameterLimit: options.parameterLimit,
        parseArrays: !!options.parseArrays,
        strictDepth: !!options.strictDepth,
        strictNullHandling: !!options.strictNullHandling,
        throwOnLimitExceeded: !!options.throwOnLimitExceeded,
    };
}

function normalizeEncodeOptions(options) {
    return {
        addQueryPrefix: !!options.addQueryPrefix,
        allowDots: !!options.allowDots,
        allowEmptyArrays: !!options.allowEmptyArrays,
        arrayFormat: options.arrayFormat,
        charset: options.charset,
        charsetSentinel: !!options.charsetSentinel,
        commaRoundTrip: !!options.commaRoundTrip,
        delimiter: options.delimiter,
        encode: !!options.encode,
        encodeDotInKeys: !!options.encodeDotInKeys,
        encodeValuesOnly: !!options.encodeValuesOnly,
        filter: Array.isArray(options.filter) ? options.filter : undefined,
        format: options.format,
        skipNulls: !!options.skipNulls,
        sort:
            options.sort === "lexicographicAsc"
                ? (left, right) => String(left).localeCompare(String(right))
                : undefined,
        strictNullHandling: !!options.strictNullHandling,
    };
}

function errorKind(error) {
    const message = String((error && error.message) || error || "");
    if (message.includes("Parameter limit exceeded")) {
        return "parameter_limit_exceeded";
    }
    if (message.includes("Array limit exceeded")) {
        return "list_limit_exceeded";
    }
    if (message.includes("Input depth exceeded")) {
        return "depth_exceeded";
    }
    if (message.includes("delimiter must not be empty")) {
        return "empty_delimiter";
    }
    return "unknown";
}

function main() {
    const payload = JSON.parse(fs.readFileSync(0, "utf8"));

    try {
        let value;
        if (payload.mode === "decode") {
            value = qs.parse(payload.input, normalizeDecodeOptions(payload.options || {}));
        } else if (payload.mode === "encode") {
            value = qs.stringify(payload.input, normalizeEncodeOptions(payload.options || {}));
        } else {
            throw new Error(`unsupported mode: ${payload.mode}`);
        }

        process.stdout.write(JSON.stringify({status: "ok", value}));
    } catch (error) {
        process.stdout.write(
            JSON.stringify({
                status: "error",
                kind: errorKind(error),
                message: String((error && error.message) || error || ""),
            }),
        );
    }
}

main();
