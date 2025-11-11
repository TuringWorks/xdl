//! # XDL Standard Library
//!
//! Built-in functions and procedures for XDL

pub mod array;
pub mod graphics; // Full implementation modules
mod graphics_procs; // Procedure wrappers
pub mod io;
pub mod math;
pub mod ml;
pub mod python;
pub mod statistics;
pub mod string;
pub mod system;

// Re-export graphics callback registration for GUI
pub use graphics_procs::{register_gui_plot_callback, register_gui_image_callback};

use xdl_core::{XdlResult, XdlValue};

/// Standard library function registry
pub struct StandardLibrary {
    // TODO: Function registry
}

impl StandardLibrary {
    pub fn new() -> Self {
        Self {
            // TODO: Initialize standard library
        }
    }

    /// Call a XDL procedure
    pub fn call_procedure(&self, name: &str, args: &[XdlValue]) -> XdlResult<XdlValue> {
        match name.to_uppercase().as_str() {
            // Graphics procedures - Basic plotting
            "PLOT" => graphics_procs::plot(args),
            "OPLOT" => graphics_procs::oplot(args),
            "PLOTS" => graphics_procs::plots(args),
            "XYOUTS" => graphics_procs::xyouts(args),
            "AXIS" => graphics_procs::axis(args),

            // Graphics procedures - 2D shapes
            "POLYFILL" => graphics_procs::polyfill(args),
            "ARROW" => graphics_procs::arrow(args),
            "USERSYM" => graphics_procs::usersym(args),

            // Graphics procedures - 3D plotting
            "CONTOUR" => graphics_procs::contour(args),
            "SURFACE" => graphics_procs::surface(args),
            "SHADE_SURF" => graphics_procs::shade_surf(args),
            "SHADE_SURF_IRR" => graphics_procs::shade_surf_irr(args),
            "SURFR" => graphics_procs::surfr(args),
            "SHOW3" => graphics_procs::show3(args),
            "T3D" => graphics_procs::t3d(args),
            "SCALE3" => graphics_procs::scale3(args),
            "PLOT3D" => graphics_procs::plot3d(args),
            "ISOCONTOUR" => graphics_procs::isocontour(args),
            "ISOSURFACE" => graphics_procs::isosurface(args),

            // Graphics procedures - Image display
            "TV" => graphics_procs::tv(args),
            "TVSCL" => graphics_procs::tvscl(args),
            "TVCRS" => graphics_procs::tvcrs(args),
            "IMAGE_DISPLAY" => graphics_procs::image_display(args),

            // Graphics procedures - Window management
            "WINDOW" => graphics_procs::window(args),
            "WSET" => graphics_procs::wset(args),
            "WDELETE" => graphics_procs::wdelete(args),
            "WSHOW" => graphics_procs::wshow(args),
            "ERASE" => graphics_procs::erase(args),
            "EMPTY" => graphics_procs::empty(args),

            // Graphics procedures - Device & color
            "DEVICE" => graphics_procs::device(args),
            "LOADCT" => graphics_procs::loadct(args),

            // Graphics procedures - Interactive
            "CURSOR" => graphics_procs::cursor(args),

            // Graphics procedures - Specialized plots
            "BAR_PLOT" => graphics_procs::bar_plot(args),
            "HISTOGRAM" => graphics_procs::histogram(args),
            "PLOTERR" => graphics_procs::ploterr(args),
            "ERRPLOT" => graphics_procs::errplot(args),
            "VEL" => graphics_procs::vel(args),
            "VELOVECT" => graphics_procs::velovect(args),

            // Graphics procedures - Map projections
            "MAP_SET" => graphics_procs::map_set(args),
            "MAP_CONTINENTS" => graphics_procs::map_continents(args),
            "MAP_GRID" => graphics_procs::map_grid(args),

            // System procedures
            "HELP" => system::help(args),
            "CD" => system::cd(args),
            "SPAWN" => system::spawn(args),
            "CALL_PROCEDURE" => system::call_procedure(args),
            "DEFSYSV" => system::defsysv(args),
            "@" => system::execute_batch(args),
            ".COMPILE" => system::compile_pro(args),
            ".CONTINUE" => system::continue_execution(args),
            "CATCH" => system::catch_error(args),

            // I/O procedures
            "FREE_LUN" => io::free_lun(args),
            "OPEN" => io::open_file(args),
            "OPENR" => io::openr(args),
            "OPENW" => io::openw(args),
            "OPENU" => io::openu(args),
            "CLOSE" => io::close_file(args),
            "WRITEF" => io::writef(args),
            "PRINTF" => io::printf(args),

            _ => Err(xdl_core::XdlError::RuntimeError(format!(
                "Unknown procedure: {}",
                name
            ))),
        }
    }

    /// Call a XDL function
    pub fn call_function(&self, name: &str, args: &[XdlValue]) -> XdlResult<XdlValue> {
        match name.to_uppercase().as_str() {
            // Trigonometric functions
            "SIN" => math::sin(args),
            "COS" => math::cos(args),
            "TAN" => math::tan(args),
            "ASIN" => math::asin(args),
            "ACOS" => math::acos(args),
            "ATAN" => math::atan(args),

            // Exponential and logarithmic functions
            "EXP" => math::exp(args),
            "ALOG" | "LN" => math::log(args),
            "ALOG10" => math::log10(args),
            "SQRT" => math::sqrt(args),

            // Other math functions
            "ABS" => math::abs(args),
            "FLOOR" => math::floor(args),
            "CEIL" => math::ceil(args),
            "ROUND" => math::round(args),

            // Array generation functions
            "FINDGEN" => math::findgen(args),
            "INDGEN" => math::indgen(args),
            "RANDOMU" => math::randomu(args),

            // Signal processing
            "FFT" => math::fft(args),

            // Array creation functions
            "BYTARR" => array::bytarr(args),
            "INTARR" => array::intarr(args),
            "LONARR" => array::lonarr(args),
            "FLTARR" => array::fltarr(args),
            "DBLARR" => array::dblarr(args),
            "STRARR" => array::strarr(args),

            "N_ELEMENTS" => array::n_elements(args),
            "WHERE" => array::where_func(args),

            // Array manipulation functions
            "REFORM" => array::reform_func(args),
            "TRANSPOSE" => array::transpose_func(args),

            // Array statistics functions
            "MIN" => array::min_func(args),
            "MAX" => array::max_func(args),
            "MEAN" => array::mean_func(args),
            "TOTAL" => array::total_func(args),
            "REVERSE" => array::reverse_func(args),
            "SORT" => array::sort_func(args),

            // Moving average functions
            "SMOOTH" => array::smooth_func(args),
            "MOVING_AVERAGE" => array::moving_average_func(args),
            "WMA" => array::wma_func(args),
            "EMA" => array::ema_func(args),
            "CUMULATIVE_AVERAGE" => array::cumulative_average_func(args),

            // Statistical functions
            "VARIANCE" => statistics::variance(args),
            "STDDEV" => statistics::stddev(args),
            "MEDIAN" => statistics::median(args),
            "MOMENT" => statistics::moment(args),
            "MEANABSDEV" => statistics::meanabsdev(args),
            "SKEWNESS" => statistics::skewness(args),
            "KURTOSIS" => statistics::kurtosis(args),

            // Probability density functions
            "GAUSS_PDF" => statistics::gauss_pdf(args),
            "T_PDF" => statistics::t_pdf(args),
            "CHISQR_PDF" => statistics::chisqr_pdf(args),

            // I/O functions
            "PRINT" => io::print(args),
            "GET_LUN" => io::get_lun(args),
            "FILEPATH" => io::filepath(args),
            "READ_JPEG" => io::read_jpeg(args),
            "READF" => io::readf(args),

            // Data structure functions
            "HASH" => create_hash(args),

            // String functions
            "STRLEN" => string::strlen(args),
            "STRPOS" => string::strpos(args),
            "STRMID" => string::strmid(args),
            "STRUPCASE" => string::strupcase(args),
            "STRLOWCASE" => string::strlowcase(args),
            "STRING" => string::string_fn(args),

            // Python integration functions
            "PYTHON_IMPORT" => python::python_import(args),
            "PYTHON_CALL" => python::python_call(args),
            "PYTHON_CALL_KW" => python::python_call_kw(args),

            // Machine Learning functions
            "XDLML_PARTITION" => ml::xdlml_partition(args),
            "XDLML_SHUFFLE" => ml::xdlml_shuffle(args),
            "XDLML_LINEAR_NORMALIZER" => ml::xdlml_linear_normalizer(args),
            "XDLML_RANGE_NORMALIZER" => ml::xdlml_range_normalizer(args),
            "XDLML_VARIANCE_NORMALIZER" => ml::xdlml_variance_normalizer(args),
            "XDLML_TANH_NORMALIZER" => ml::xdlml_tanh_normalizer(args),
            "XDLML_UNIT_NORMALIZER" => ml::xdlml_unit_normalizer(args),
            "XDLML_KMEANS" => ml::xdlml_kmeans(args),

            // Activation functions (Phase ML-2)
            "XDLMLAF_IDENTITY" => ml::xdlmlaf_identity(args),
            "XDLMLAF_BINARYSTEP" => ml::xdlmlaf_binarystep(args),
            "XDLMLAF_LOGISTIC" => ml::xdlmlaf_logistic(args),
            "XDLMLAF_TANH" => ml::xdlmlaf_tanh(args),
            "XDLMLAF_RELU" => ml::xdlmlaf_relu(args),
            "XDLMLAF_PRELU" => ml::xdlmlaf_prelu(args),
            "XDLMLAF_ELU" => ml::xdlmlaf_elu(args),
            "XDLMLAF_SOFTPLUS" => ml::xdlmlaf_softplus(args),
            "XDLMLAF_SOFTSIGN" => ml::xdlmlaf_softsign(args),
            "XDLMLAF_SOFTMAX" => ml::xdlmlaf_softmax(args),
            "XDLMLAF_ARCTAN" => ml::xdlmlaf_arctan(args),
            "XDLMLAF_GAUSSIAN" => ml::xdlmlaf_gaussian(args),
            "XDLMLAF_SINC" => ml::xdlmlaf_sinc(args),
            "XDLMLAF_SINUSOID" => ml::xdlmlaf_sinusoid(args),
            "XDLMLAF_BENTIDENTITY" => ml::xdlmlaf_bentidentity(args),
            "XDLMLAF_ISRU" => ml::xdlmlaf_isru(args),
            "XDLMLAF_ISRLU" => ml::xdlmlaf_isrlu(args),
            "XDLMLAF_SOFTEXPONENTIAL" => ml::xdlmlaf_softexponential(args),

            // Loss functions (Phase ML-2)
            "XDLMLLF_MEANSQUAREDERROR" => ml::xdlmllf_meansquarederror(args),
            "XDLMLLF_MEANABSOLUTEERROR" => ml::xdlmllf_meanabsoluteerror(args),
            "XDLMLLF_CROSSENTROPY" => ml::xdlmllf_crossentropy(args),
            "XDLMLLF_HUBER" => ml::xdlmllf_huber(args),
            "XDLMLLF_LOGCOSH" => ml::xdlmllf_logcosh(args),

            // Optimizers (Phase ML-3)
            "XDLMLOPT_GRADIENTDESCENT" => ml::xdlmlopt_gradientdescent(args),
            "XDLMLOPT_MOMENTUM" => ml::xdlmlopt_momentum(args),
            "XDLMLOPT_RMSPROP" => ml::xdlmlopt_rmsprop(args),
            "XDLMLOPT_ADAM" => ml::xdlmlopt_adam(args),
            "XDLMLOPT_QUICKPROP" => ml::xdlmlopt_quickprop(args),

            // SVM Kernels (Phase ML-5)
            "XDLML_SVMLINEARKERNEL" => ml::xdlml_svmlinearkernel(args),
            "XDLML_SVMPOLYNOMIALKERNEL" => ml::xdlml_svmpolynomialkernel(args),
            "XDLML_SVMRADIALKERNEL" => ml::xdlml_svmradialkernel(args),
            "XDLML_SVMSIGMOIDKERNEL" => ml::xdlml_svmsigmoidkernel(args),

            // Model Evaluation (Phase ML-6)
            "XDLML_TESTCLASSIFIER" => ml::xdlml_testclassifier(args),

            // Classifier Models (Phase ML-6)
            "XDLML_SOFTMAX" => ml::xdlml_softmax(args),

            // Neural Network Models (Phase ML-4)
            "XDLML_FEEDFORWARDNEURALNETWORK" => ml::xdlml_feedforwardneuralnetwork(args),
            "XDLML_AUTOENCODER" => ml::xdlml_autoencoder(args),

            // SVM Models (Phase ML-5)
            "XDLML_SUPPORTVECTORMACHINECLASSIFICATION" => {
                ml::xdlml_supportvectormachineclassification(args)
            }
            "XDLML_SUPPORTVECTORMACHINEREGRESSION" => {
                ml::xdlml_supportvectormachineregression(args)
            }

            // Cross-Validation Utilities (Phase ML-7)
            "XDLML_KFOLD" => ml::xdlml_kfold(args),
            "XDLML_STRATIFIEDKFOLD" => ml::xdlml_stratifiedkfold(args),
            "XDLML_LEAVEONEOUT" => ml::xdlml_leaveoneout(args),

            // Regularization Layers (Phase ML-8)
            "XDLML_BATCHNORMALIZATION" => ml::xdlml_batchnormalization(args),
            "XDLML_DROPOUT" => ml::xdlml_dropout(args),

            // Convolutional & Pooling Layers (Phase ML-9)
            "XDLML_CONV1D" => ml::xdlml_conv1d(args),
            "XDLML_MAXPOOLING1D" => ml::xdlml_maxpooling1d(args),
            "XDLML_AVERAGEPOOLING1D" => ml::xdlml_averagepooling1d(args),

            // Recurrent Layers (Phase ML-10)
            "XDLML_SIMPLERNN" => ml::xdlml_simplernn(args),
            "XDLML_SEQUENCEMEAN" => ml::xdlml_sequencemean(args),

            // Matrix Operations (Phase ML-11)
            "XDLML_MATMUL" => ml::xdlml_matmul(args),
            "XDLML_RESHAPE" => ml::xdlml_reshape(args),
            "XDLML_TRANSPOSE" => ml::xdlml_transpose(args),

            // Convolutional Layers (Phase ML-11)
            "XDLML_CONV2D" => ml::xdlml_conv2d(args),
            "XDLML_MAXPOOLING2D" => ml::xdlml_maxpooling2d(args),

            // LSTM Layer (Phase ML-11)
            "XDLML_LSTM" => ml::xdlml_lstm(args),

            _ => Err(xdl_core::XdlError::FunctionNotFound(name.to_string())),
        }
    }
}

impl Default for StandardLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a hash table (dictionary) from key-value pairs
fn create_hash(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // For now, return a simple placeholder since XdlValue doesn't have a Hash variant
    // In a full implementation, this would create a proper hash table structure
    if args.is_empty() {
        // Return an empty hash representation
        Ok(XdlValue::String("{}".to_string()))
    } else {
        // For simplicity, convert arguments to a string representation
        // In practice, this would build a proper hash table from key-value pairs
        let mut result = String::from("{");
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&arg.to_string_repr());
        }
        result.push('}');
        Ok(XdlValue::String(result))
    }
}
