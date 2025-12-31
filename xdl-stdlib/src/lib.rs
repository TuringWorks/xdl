//! # XDL Standard Library
//!
//! Built-in functions and procedures for XDL

pub mod amp; // Accelerated Math Processing (SIMD/GPU)
pub mod array;
mod charting_procs; // ECharts charting procedures
pub mod complex;
pub mod data_structures; // Pointers, objects, lists, hashes
pub mod dialog; // Dialog functions for user interaction
pub mod gpu_array; // GPU-accelerated array operations
pub mod graphics; // Full implementation modules
mod graphics_procs; // Procedure wrappers
pub mod image; // Image processing
pub mod image_io; // Image file I/O (PNG, JPEG, TIFF, etc.)
pub mod io;
pub mod linalg; // Linear algebra
pub mod map; // Map projections
pub mod math;
pub mod matlab_compat; // MATLAB compatibility functions
pub mod ml;
#[cfg(feature = "python")]
pub mod python;
pub mod scientific_io; // Scientific data formats (FITS, HDF5, NetCDF)
pub mod signal; // Signal processing
pub mod statistics;
pub mod string;
pub mod system;
pub mod viz3d; // 3D volume visualization
pub mod viz3d_advanced; // Advanced 3D visualization (isosurface, streamlines)
pub mod widget; // Widget/GUI functions

// Data Science modules (feature-gated)
#[cfg(feature = "dataframes")]
pub mod polars_df; // Polars DataFrames (Pandas alternative)

// Linfa ML (scikit-learn alternative)
#[cfg(feature = "ml")]
pub mod linfa_ml;

#[cfg(feature = "rustpython")]
pub mod rustpython_interp; // RustPython interpreter

// Re-export graphics callback registration for GUI
pub use graphics_procs::{register_gui_image_callback, register_gui_plot_callback};

use std::collections::HashMap;
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
        self.call_procedure_with_keywords(name, args, &HashMap::new())
    }

    /// Call a XDL procedure with keyword arguments
    pub fn call_procedure_with_keywords(
        &self,
        name: &str,
        args: &[XdlValue],
        keywords: &HashMap<String, XdlValue>,
    ) -> XdlResult<XdlValue> {
        match name.to_uppercase().as_str() {
            // Graphics procedures - Basic plotting
            "PLOT" => graphics_procs::plot(args),
            "OPLOT" => graphics_procs::oplot(args),
            "PLOTS" => graphics_procs::plots(args),
            "XYOUTS" => graphics_procs::xyouts(args),
            "AXIS" => graphics_procs::axis(args),
            "SET_PLOT_BACKEND" => graphics_procs::set_plot_backend_proc(args),

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
            "MAP_SET" => map::map_set(args, keywords),
            "MAP_CONTINENTS" => map::map_continents(args, keywords),
            "MAP_GRID" => map::map_grid(args, keywords),

            // Graphics procedures - Advanced visualization
            "RENDER_COLORMAP" => graphics_procs::render_colormap(args),
            "DEM_RENDER" => graphics_procs::dem_render(args),
            "HILLSHADE" => graphics_procs::hillshade_proc(args),
            "QUIVER" => graphics_procs::quiver_proc(args),

            // Graphics procedures - Plotting utilities
            "OCONTOUR" => graphics_procs::ocontour(args),
            "WARP_TRI" => graphics_procs::warp_tri(args),
            "POLYWARP" => graphics_procs::polywarp(args),
            "POLY_2D" => graphics_procs::poly_2d(args),
            "ANNOTATE" => graphics_procs::annotate(args),
            "RDPIX" => graphics_procs::rdpix(args),
            "PROFILES" => graphics_procs::profiles(args),
            "TVLCT" => graphics_procs::tvlct(args),
            "XYOUTS_EXTENDED" => graphics_procs::xyouts_extended(args),
            "LEGEND" => graphics_procs::legend(args),
            "COLORBAR" => graphics_procs::colorbar(args),

            // Charting procedures - ECharts integration (interactive HTML)
            "CHART_PLOT" => charting_procs::plot(args),
            "CHART_SCATTER" => charting_procs::scatter(args),
            "CHART_BAR" => charting_procs::bar(args),
            "CHART_CONTOUR" => charting_procs::contour(args),
            "CHART_SHADE_SURF" => charting_procs::shade_surf(args),
            "CHART_PLOT3D" => charting_procs::plot3d(args),
            "SURFACE3D" => charting_procs::surface3d(args),
            "SCATTER3D" => charting_procs::scatter3d(args),

            // VIZ3D procedures - 3D volume visualization
            "VIZ3D_INIT" => viz3d::viz3d_init(args, keywords),
            "VIZ3D_VOLUME" => viz3d::viz3d_volume(args, keywords),
            "VIZ3D_COLORMAP" => viz3d::viz3d_colormap(args, keywords),
            "VIZ3D_CAMERA" => viz3d::viz3d_camera(args, keywords),
            "VIZ3D_RENDER" => viz3d::viz3d_render(args, keywords),
            "VIZ3D_TRANSFER" => viz3d::viz3d_transfer(args, keywords),
            "VIZ3D_LIGHT" => viz3d::viz3d_light(args, keywords),
            "VIZ3D_ISOSURFACE" => viz3d::viz3d_isosurface(args, keywords),

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
            "WAIT" => system::wait(args),
            "STOP" => system::stop(args),

            // Time functions
            "SYSTIME" => system::systime(args),
            "JULDAY" => system::julday(args),
            "CALDAT" => system::caldat(args),
            "TIC" => system::tic(args),
            "TOC" => system::toc(args),
            // Additional time functions (Phase 14)
            "WEEKDAY" => system::weekday(args),
            "BIN_DATE" => system::bin_date(args),
            "TIMESTAMP" => system::timestamp(args),
            "TIMEGEN" => system::timegen(args),
            "DAYOFYEAR" => system::dayofyear(args),
            "JS2JD" => system::js2jd(args),

            // Structure and type functions
            "N_TAGS" => system::n_tags(args),
            "TAG_NAMES" => system::tag_names(args),
            "SIZE" => system::size_func(args),
            "ISA" => system::isa(args),

            // System control functions (Phase 18)
            "MEMORY" => system::memory(args),
            "ROUTINE_INFO" => system::routine_info(args),
            "N_PARAMS" => system::n_params(args),
            "SCOPE_VARNAME" => system::scope_varname(args),
            "SCOPE_LEVEL" => system::scope_level(args),
            "SCOPE_TRACEBACK" => system::scope_traceback(args),
            "EXIT" => system::exit_session(args),
            "RETALL" => system::retall(args),
            "MESSAGE" => system::message(args),
            "ON_ERROR" => system::on_error(args),
            "EXECUTE" => system::execute(args),

            // Pointer and Object Management (Phase 15)
            "PTR_NEW" => data_structures::ptr_new(args),
            "PTR_VALID" => data_structures::ptr_valid(args),
            "PTR_FREE" => data_structures::ptr_free(args),
            "PTR_DEREF" => data_structures::ptr_deref(args),
            "OBJ_NEW" => data_structures::obj_new(args),
            "OBJ_VALID" => data_structures::obj_valid(args),
            "OBJ_DESTROY" => data_structures::obj_destroy(args),
            "OBJ_CLASS" => data_structures::obj_class(args),
            "OBJ_ISA" => data_structures::obj_isa(args),
            "OBJ_HASMETHOD" => data_structures::obj_hasmethod(args),
            "OBJ_PARENT" => data_structures::obj_parent(args),
            "CALL_METHOD" => data_structures::call_method(args),
            "SETPROPERTY" => data_structures::setproperty(args),
            "GETPROPERTY" => data_structures::getproperty(args),

            // Data Structures (Phase 16)
            "LIST" => data_structures::list(args),
            "LIST_ADD" => data_structures::list_add(args),
            "LIST_COUNT" => data_structures::list_count(args),
            "ORDEREDHASH" => data_structures::orderedhash(args),
            "DICTIONARY" => data_structures::dictionary(args),
            "CREATE_STRUCT" => data_structures::create_struct(args),
            "STRUCT_ASSIGN" => data_structures::struct_assign(args),
            "HEAP_GC" => data_structures::heap_gc(args),
            "HEAP_FREE" => data_structures::heap_free(args),

            // Signal processing procedures
            "A_CORRELATE" => signal::a_correlate(args),
            "C_CORRELATE" => signal::c_correlate(args),
            "DIGITAL_FILTER" => signal::digital_filter(args),
            "HILBERT" => signal::hilbert(args),
            "MEDIAN_FILTER" => signal::median_filter(args),

            // Image processing procedures
            "CONVOL" => image::convol(args),
            "DILATE" => image::dilate(args),
            "ERODE" => image::erode(args),
            "SOBEL" => image::sobel(args),
            "ROBERTS" => image::roberts(args),
            "PREWITT" => image::prewitt(args),
            "GAUSSIAN_FILTER" => image::gaussian_filter(args),
            "THRESHOLD" => image::threshold(args),

            // Linear algebra procedures
            "IDENTITY" => linalg::identity(args),
            "INVERT" => linalg::invert(args),
            "DETERM" => linalg::determ(args),
            "CROSSP" => linalg::crossp(args),
            "DOTP" => linalg::dotp(args),
            "NORM" => linalg::norm(args),
            "DIAGONAL" => linalg::diagonal(args),
            "TRACE" => linalg::trace(args),
            "SVDC" => linalg::svdc(args),
            "LA_EIGENVAL" => linalg::la_eigenval(args),
            "LUDC" => linalg::ludc(args),
            "LUSOL" => linalg::lusol(args),

            // I/O procedures
            "FREE_LUN" => io::free_lun(args),
            "OPEN" => io::open_file(args),
            "OPENR" => io::openr(args),
            "OPENW" => io::openw(args),
            "OPENU" => io::openu(args),
            "CLOSE" => io::close_file(args),
            "WRITEF" => io::writef(args),
            "PRINTF" => io::printf(args),

            // Widget/GUI procedures
            "WIDGET_CONTROL" => widget::widget_control(args, keywords),
            "XMANAGER" => widget::xmanager(args, keywords),

            _ => Err(xdl_core::XdlError::RuntimeError(format!(
                "Unknown procedure: {}",
                name
            ))),
        }
    }

    /// Call a XDL function
    pub fn call_function(&self, name: &str, args: &[XdlValue]) -> XdlResult<XdlValue> {
        self.call_function_with_keywords(name, args, &HashMap::new())
    }

    /// Call a XDL function with keyword arguments
    pub fn call_function_with_keywords(
        &self,
        name: &str,
        args: &[XdlValue],
        keywords: &HashMap<String, XdlValue>,
    ) -> XdlResult<XdlValue> {
        match name.to_uppercase().as_str() {
            // Trigonometric functions
            "SIN" => math::sin(args),
            "COS" => math::cos(args),
            "TAN" => math::tan(args),
            "ASIN" => math::asin(args),
            "ACOS" => math::acos(args),
            "ATAN" => {
                // ATAN can take 1 or 2 arguments (ATAN(x) or ATAN(y, x))
                if args.len() == 2 {
                    math::atan2(args)
                } else {
                    math::atan(args)
                }
            }

            // Hyperbolic functions
            "SINH" => math::sinh(args),
            "COSH" => math::cosh(args),
            "TANH" => math::tanh(args),
            "ASINH" => math::asinh(args),
            "ACOSH" => math::acosh(args),
            "ATANH" => math::atanh(args),

            // Special math functions
            "ERF" => math::erf(args),
            "ERFC" => math::erfc(args),
            "GAMMA" => math::gamma_func(args),
            "LNGAMMA" => math::lngamma(args),
            "FACTORIAL" => math::factorial(args),
            "BESELJ" => math::beselj(args),
            "BESELY" => math::besely(args),
            "BESELI" => math::beseli(args),
            "BESELK" => math::beselk(args),

            // Additional math functions (Phase 6)
            "PRIME" => math::prime(args),
            "PRIMES" => math::primes(args),
            "BINOMIAL" => math::binomial(args),
            "GCD" => math::gcd(args),
            "LCM" => math::lcm(args),
            "BETA" => math::beta(args),
            "DERIV" => math::deriv(args),
            "INT_TABULATED" => math::int_tabulated(args),
            "POLY" => math::poly(args),
            "PRODUCT" => math::product(args),
            "POW" => math::pow(args),
            "ALOG2" => math::alog2(args),
            "FINITE" => math::finite(args),
            "CHECK_MATH" => math::check_math(args),
            "MACHAR" => math::machar(args),

            // Additional math utilities
            "SIGN" => math::sign(args),
            "HYPOT" => math::hypot(args),
            "ISNAN" => math::isnan_func(args),
            "ISINF" => math::isinf_func(args),
            "TRUNC" => math::trunc_func(args),
            "FRAC" => math::frac_func(args),
            "SIGNUM" => math::signum_func(args),
            "CBRT" => math::cbrt_func(args),
            "COPYSIGN" => math::copysign_func(args),
            "FDIM" => math::fdim_func(args),
            "FMA" => math::fma_func(args),
            "REMAINDER" => math::remainder_func(args),
            "LDEXP" => math::ldexp_func(args),
            "FREXP" => math::frexp_func(args),

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
            "FIX" => math::fix(args),
            "INT" => math::fix_func(args),
            "LONG" => math::long_func(args),
            "FLOAT" => math::float_func(args),
            "DOUBLE" => math::double_func(args),
            "BYTE" => math::byte_func(args),
            "UINT" => math::uint_func(args),
            "ULONG" => math::ulong_func(args),
            "LONG64" => math::long64_func(args),
            "ULONG64" => math::ulong64_func(args),

            // Array generation functions (with keyword support for START/INCREMENT)
            "FINDGEN" => math::findgen_with_keywords(args, keywords),
            "DINDGEN" => math::dindgen_with_keywords(args, keywords),
            "BINDGEN" => math::bindgen_with_keywords(args, keywords),
            "CINDGEN" => math::cindgen_with_keywords(args, keywords),
            "DCINDGEN" => math::dcindgen_with_keywords(args, keywords),
            "INDGEN" => math::indgen_with_keywords(args, keywords),
            "LINDGEN" => math::lindgen_with_keywords(args, keywords),
            "L64INDGEN" => math::l64indgen_with_keywords(args, keywords),
            "SINDGEN" => math::sindgen_with_keywords(args, keywords),
            "UINDGEN" => math::uindgen_with_keywords(args, keywords),
            "UL64INDGEN" => math::ul64indgen_with_keywords(args, keywords),
            "ULINDGEN" => math::ulindgen_with_keywords(args, keywords),
            "MAKE_ARRAY" => math::make_array(args, keywords),
            "NORMALIZE" => math::normalize_func(args, keywords),
            "RANDOMU" => math::randomu(args),
            "RANDOMN" => math::randomn(args),
            "MESHGRID" => math::meshgrid(args),

            // MATLAB compatibility functions
            "LINSPACE" => matlab_compat::linspace(args),
            "LOGSPACE" => matlab_compat::logspace(args),
            "REPMAT" => matlab_compat::repmat(args),
            "SQUEEZE" => matlab_compat::squeeze(args),
            "NDGRID" => matlab_compat::ndgrid(args),
            "INTERP1" => matlab_compat::interp1(args),

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
            "SHIFT" => array::shift_func(args),
            "ROTATE" => array::rotate_func(args),
            "REPLICATE" => array::replicate_func(args),
            // MAKE_ARRAY moved to keyword-aware section above
            "ARRAY_EQUAL" => array::array_equal_func(args),
            "UNIQ" => array::uniq_func(args),
            "HISTOGRAM" => array::histogram_func(args),
            "REBIN" => array::rebin_func(args),
            "CONGRID" => array::congrid_func(args),

            // Additional array utility functions
            "CUMSUM" => array::cumsum_func(args, keywords),
            "CUMPROD" => array::cumprod_func(args, keywords),
            "ARGMIN" => array::argmin_func(args, keywords),
            "ARGMAX" => array::argmax_func(args, keywords),
            "DIFF" => array::diff_func(args, keywords),
            "APPEND" => array::append_func(args, keywords),
            "ANY" => array::any_func(args, keywords),
            "ALL" => array::all_func(args, keywords),
            "FLATTEN" => array::flatten_func(args, keywords),
            "NONZERO" => array::nonzero_func(args, keywords),
            "CLIP" => array::clip_func(args, keywords),
            "ARANGE" => array::arange_func(args, keywords),
            "SEARCHSORTED" => array::searchsorted_func(args, keywords),
            "DIGITIZE" => array::digitize_func(args, keywords),
            "TILE" => array::tile_func(args, keywords),

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

            // Fitting functions
            "LINFIT" => statistics::linfit(args),
            "POLY_FIT" => statistics::poly_fit(args),
            "REGRESS" => statistics::regress(args),
            "CORRELATE" => statistics::correlate(args),
            "R_CORRELATE" => statistics::r_correlate(args),
            "LADFIT" => statistics::ladfit(args),
            "SVDFIT" => statistics::svdfit(args),
            "CURVEFIT" => statistics::curvefit(args),

            // Additional statistical functions
            "PERCENTILES" => statistics::percentiles(args),
            "ROBUST_MEAN" => statistics::robust_mean(args),
            "TRIMMED_MEAN" => statistics::trimmed_mean(args),
            "RESISTANT_MEAN" => statistics::resistant_mean(args),
            "RANDOM_POISSON" => statistics::random_poisson(args),
            "MODE" => statistics::mode(args),
            "HISTOGRAM2D" => statistics::histogram2d(args, keywords),

            // Interpolation functions
            "INTERPOL" => statistics::interpol(args),
            "SPLINE" => statistics::spline(args),
            "BILINEAR" => statistics::bilinear(args),

            // I/O functions
            "PRINT" => io::print(args),
            "GET_LUN" => io::get_lun(args),
            "FILEPATH" => io::filepath(args),
            "READ_JPEG" => io::read_jpeg(args),
            "READF" => io::readf(args),

            // File system functions
            "FILE_TEST" => io::file_test(args),
            "FILE_INFO" => io::file_info(args),
            "FILE_SEARCH" => io::file_search(args),
            "FILE_MKDIR" => io::file_mkdir(args),
            "FILE_DELETE" => io::file_delete(args),
            "FILE_COPY" => io::file_copy(args),
            "FILE_MOVE" => io::file_move(args),
            "EOF" => io::eof_func(args),
            "FLUSH" => io::flush_func(args),
            "CD" => io::cd_func(args),
            "GETENV" => io::getenv(args),
            "SETENV" => io::setenv(args),
            "SPAWN" => io::spawn_func(args),

            // Additional file I/O functions (Phase 9)
            "FILE_EXPAND_PATH" => io::file_expand_path(args),
            "FILE_SAME" => io::file_same(args),
            "FILE_CHMOD" => io::file_chmod(args),
            "FINDFILE" => io::findfile(args),
            "FILE_BASENAME" => io::file_basename(args),
            "FILE_DIRNAME" => io::file_dirname(args),
            "FILE_LINES" => io::file_lines(args),
            "POINT_LUN" => io::point_lun(args),
            "READU" => io::readu(args),
            "WRITEU" => io::writeu(args),
            "ASSOC" => io::assoc(args),

            // Image I/O functions (Phase 10)
            "READ_PNG" => image_io::read_png(args),
            "WRITE_PNG" => image_io::write_png(args),
            "WRITE_JPEG" => image_io::write_jpeg(args),
            "READ_TIFF" => image_io::read_tiff(args),
            "WRITE_TIFF" => image_io::write_tiff(args),
            "READ_BMP" => image_io::read_bmp(args),
            "WRITE_BMP" => image_io::write_bmp(args),
            "READ_GIF" => image_io::read_gif(args),
            "WRITE_GIF" => image_io::write_gif(args),
            "READ_IMAGE" => image_io::read_image(args),
            "WRITE_IMAGE" => image_io::write_image(args),
            "QUERY_IMAGE" => image_io::query_image(args),

            // Time functions
            "SYSTIME" => system::systime(args),
            "JULDAY" => system::julday(args),
            "CALDAT" => system::caldat(args),
            "TIC" => system::tic(args),
            "TOC" => system::toc(args),
            // Additional time functions (Phase 14)
            "WEEKDAY" => system::weekday(args),
            "BIN_DATE" => system::bin_date(args),
            "TIMESTAMP" => system::timestamp(args),
            "TIMEGEN" => system::timegen(args),
            "DAYOFYEAR" => system::dayofyear(args),
            "JS2JD" => system::js2jd(args),

            // Structure and type functions
            "N_TAGS" => system::n_tags(args),
            "TAG_NAMES" => system::tag_names(args),
            "SIZE" => system::size_func(args),
            "ISA" => system::isa(args),

            // System control functions (Phase 18)
            "MEMORY" => system::memory(args),
            "ROUTINE_INFO" => system::routine_info(args),
            "N_PARAMS" => system::n_params(args),
            "SCOPE_VARNAME" => system::scope_varname(args),
            "SCOPE_LEVEL" => system::scope_level(args),
            "SCOPE_TRACEBACK" => system::scope_traceback(args),

            // Path and system utilities
            "PATH_SEP" => system::path_sep(args),
            "ADD_SLASH" => system::add_slash(args),
            "GET_SCREEN_SIZE" => system::get_screen_size(args),
            "GETENV_ALL" => system::getenv_all(args),
            "UNSETENV" => system::unsetenv(args),
            "CPU" => system::cpu(args),
            "HOSTNAME" => system::hostname(args),
            "TEMPORARY" => system::temporary(args),
            "SLEEP" => system::sleep(args),
            "VERSION" => system::version(args),
            "PLATFORM" => system::platform(args),
            "IS_WINDOWS" => system::is_windows(args),
            "IS_MACOS" => system::is_macos(args),
            "IS_LINUX" => system::is_linux(args),
            "WHICH" => system::which(args),

            // Dialog functions
            "DIALOG_MESSAGE" => dialog::dialog_message(args, keywords),
            "DIALOG_PICKFILE" => dialog::dialog_pickfile(args, keywords),
            "DIALOG_PRINTERSETUP" => dialog::dialog_printersetup(args, keywords),
            "DIALOG_READ_TEXT" => dialog::dialog_read_text(args, keywords),

            // Map projection functions
            "CONVERT_COORD" => map::convert_coord(args, keywords),
            "MAP_STRUCT" => map::map_struct(args),

            // Advanced 3D visualization functions
            "ISOSURFACE" => viz3d_advanced::isosurface(args, keywords),
            "SHADE_VOLUME" => viz3d_advanced::shade_volume(args, keywords),
            "PARTICLE_TRACE" => viz3d_advanced::particle_trace(args, keywords),
            "STREAMLINE" => viz3d_advanced::streamline(args, keywords),
            "VOXEL_PROJ" => viz3d_advanced::voxel_proj(args, keywords),
            "POLYSHADE" => viz3d_advanced::polyshade(args, keywords),

            // Graphics utility functions (also registered as procedures)
            "WARP_TRI" => graphics_procs::warp_tri(args),
            "POLYWARP" => graphics_procs::polywarp(args),
            "POLY_2D" => graphics_procs::poly_2d(args),
            "RDPIX" => graphics_procs::rdpix(args),
            "PROFILES" => graphics_procs::profiles(args),

            // Widget/GUI functions
            "WIDGET_BASE" => widget::widget_base(args, keywords),
            "WIDGET_BUTTON" => widget::widget_button(args, keywords),
            "WIDGET_SLIDER" => widget::widget_slider(args, keywords),
            "WIDGET_TEXT" => widget::widget_text(args, keywords),
            "WIDGET_LABEL" => widget::widget_label(args, keywords),
            "WIDGET_DRAW" => widget::widget_draw(args, keywords),
            "WIDGET_LIST" => widget::widget_list(args, keywords),
            "WIDGET_DROPLIST" => widget::widget_droplist(args, keywords),
            "WIDGET_CONTROL" => widget::widget_control(args, keywords),
            "WIDGET_INFO" => widget::widget_info(args, keywords),
            "WIDGET_EVENT" => widget::widget_event(args, keywords),
            "XMANAGER" => widget::xmanager(args, keywords),
            // Additional widget functions
            "WIDGET_TABLE" => widget::widget_table(args, keywords),
            "WIDGET_TREE" => widget::widget_tree(args, keywords),
            "WIDGET_TAB" => widget::widget_tab(args, keywords),
            "WIDGET_COMBOBOX" => widget::widget_combobox(args, keywords),
            "WIDGET_PROPERTYSHEET" => widget::widget_propertysheet(args, keywords),
            "WIDGET_DISPLAYCONTEXTMENU" => widget::widget_displaycontextmenu(args, keywords),
            // Compound widgets
            "CW_FIELD" => widget::cw_field(args, keywords),
            "CW_BGROUP" => widget::cw_bgroup(args, keywords),
            "CW_PDMENU" => widget::cw_pdmenu(args, keywords),
            // Widget utilities
            "XREGISTERED" => widget::xregistered(args, keywords),
            "XLOADCT" => widget::xloadct(args, keywords),
            "XPALETTE" => widget::xpalette(args, keywords),
            "XDISPLAYFILE" => widget::xdisplayfile(args, keywords),

            // Scientific data format functions (FITS, HDF5, NetCDF)
            "READFITS" => scientific_io::readfits(args, keywords),
            "WRITEFITS" => scientific_io::writefits(args, keywords),
            "HEADFITS" => scientific_io::headfits(args),
            "SXPAR" => scientific_io::sxpar(args),
            "FXPAR" => scientific_io::fxpar(args),
            "FXADDPAR" => scientific_io::fxaddpar(args),
            "MRDFITS" => scientific_io::mrdfits(args, keywords),
            "MWRFITS" => scientific_io::mwrfits(args, keywords),
            "FXREAD" => scientific_io::fxread(args, keywords),
            "FXWRITE" => scientific_io::fxwrite(args, keywords),
            "FITS_INFO" => scientific_io::fits_info(args, keywords),
            // File format query functions
            "QUERY_FITS" => scientific_io::query_fits(args),
            "QUERY_HDF5" => scientific_io::query_hdf5(args),
            "QUERY_NETCDF" => scientific_io::query_netcdf(args),
            "QUERY_ASCII" => scientific_io::query_ascii(args),
            "QUERY_CSV" => scientific_io::query_csv(args),
            "QUERY_JSON" => scientific_io::query_json(args),
            "QUERY_XML" => scientific_io::query_xml(args),
            // HDF5 functions
            "H5F_OPEN" => scientific_io::h5f_open(args),
            "H5F_CLOSE" => scientific_io::h5f_close(args),
            "H5D_OPEN" => scientific_io::h5d_open(args),
            "H5D_READ" => scientific_io::h5d_read(args),
            "H5D_CLOSE" => scientific_io::h5d_close(args),
            "H5D_GET_SPACE" => scientific_io::h5d_get_space(args),
            "H5D_GET_TYPE" => scientific_io::h5d_get_type(args),
            "H5G_OPEN" => scientific_io::h5g_open(args),
            "H5G_CLOSE" => scientific_io::h5g_close(args),
            "H5G_GET_NMEMBERS" => scientific_io::h5g_get_nmembers(args),
            "H5G_GET_MEMBER_NAME" => scientific_io::h5g_get_member_name(args),
            "H5A_OPEN" => scientific_io::h5a_open(args),
            "H5A_READ" => scientific_io::h5a_read(args),
            "H5A_CLOSE" => scientific_io::h5a_close(args),
            "H5A_GET_NAME" => scientific_io::h5a_get_name(args),
            "H5A_GET_NUM_ATTRS" => scientific_io::h5a_get_num_attrs(args),
            "H5S_GET_SIMPLE_EXTENT_DIMS" => scientific_io::h5s_get_simple_extent_dims(args),
            "H5S_GET_SIMPLE_EXTENT_NDIMS" => scientific_io::h5s_get_simple_extent_ndims(args),
            "H5S_CLOSE" => scientific_io::h5s_close(args),
            "H5T_GET_SIZE" => scientific_io::h5t_get_size(args),
            "H5T_CLOSE" => scientific_io::h5t_close(args),
            // NetCDF functions
            "NCDF_OPEN" => scientific_io::ncdf_open(args),
            "NCDF_CLOSE" => scientific_io::ncdf_close(args),
            "NCDF_VARGET" => scientific_io::ncdf_varget(args),
            "NCDF_INQUIRE" => scientific_io::ncdf_inquire(args),
            "NCDF_VARINQ" => scientific_io::ncdf_varinq(args),
            "NCDF_DIMINQ" => scientific_io::ncdf_diminq(args),
            "NCDF_DIMID" => scientific_io::ncdf_dimid(args),
            "NCDF_VARID" => scientific_io::ncdf_varid(args),
            "NCDF_ATTNAME" => scientific_io::ncdf_attname(args),
            "NCDF_ATTGET" => scientific_io::ncdf_attget(args),
            "NCDF_ATTINQ" => scientific_io::ncdf_attinq(args),
            "NCDF_CREATE" => scientific_io::ncdf_create(args, keywords),
            "NCDF_DIMDEF" => scientific_io::ncdf_dimdef(args),
            "NCDF_VARDEF" => scientific_io::ncdf_vardef(args),
            "NCDF_VARPUT" => scientific_io::ncdf_varput(args),
            "NCDF_ATTPUT" => scientific_io::ncdf_attput(args),
            "NCDF_CONTROL" => scientific_io::ncdf_control(args, keywords),

            // Data structure functions
            "HASH" => create_hash(args),

            // String functions
            "STRLEN" => string::strlen(args),
            "STRPOS" => string::strpos(args),
            "STRMID" => string::strmid(args),
            "STRUPCASE" => string::strupcase(args),
            "STRLOWCASE" => string::strlowcase(args),
            "STRING" => string::string_fn(args),
            "STRTRIM" => string::strtrim(args),
            "STRJOIN" => string::strjoin(args),
            "STRSPLIT" => string::strsplit(args),
            "STRCOMPRESS" => string::strcompress(args),
            "STRCMP" => string::strcmp(args),
            "STREGEX" => string::stregex(args),
            "STRREPLACE" => string::strreplace(args),
            "READS" => string::reads(args),
            "READS_STRING" => string::reads_string(args),
            "SPRINTF" => string::sprintf(args),
            "STRTOK" => string::strtok(args, keywords),
            "STRPUT" => string::strput(args),
            "STRMID_BYTES" => string::strmid_bytes(args),
            "STR_TO_BYTE" => string::str_to_byte(args),
            "STRING_FROM_BYTES" => string::string_from_bytes(args),
            "STRPOS_ALL" => string::strpos_all(args),
            "STRCOUNT" => string::strcount(args),
            "STRPAD" => string::strpad(args, keywords),
            "STRREVERSE" => string::strreverse(args),

            // Complex number functions
            "COMPLEX" => complex::complex(args),
            "REAL" => complex::real_part(args),
            "IMAGINARY" | "IMAG" => complex::imaginary_part(args),
            "CONJ" => complex::conj(args),
            // Additional complex functions (Phase 17)
            "DCOMPLEX" => complex::dcomplex(args),
            "COMPLEXARR" => complex::complexarr(args),
            "DCOMPLEXARR" => complex::dcomplexarr(args),
            "ARG" | "PHASE" => complex::arg(args),
            "COMPLEX_EXP" => complex::complex_exp(args),
            "COMPLEX_LOG" => complex::complex_log(args),
            "COMPLEX_SQRT" => complex::complex_sqrt(args),
            "COMPLEX_SIN" => complex::complex_sin(args),
            "COMPLEX_COS" => complex::complex_cos(args),
            "POLAR" => complex::polar(args),

            // Linear algebra functions
            "IDENTITY" => linalg::identity(args),
            "INVERT" => linalg::invert(args),
            "DETERM" => linalg::determ(args),
            "CROSSP" => linalg::crossp(args),
            "DOTP" => linalg::dotp(args),
            "NORM" => linalg::norm(args),
            "DIAGONAL" => linalg::diagonal(args),
            "TRACE" => linalg::trace(args),
            "SVDC" => linalg::svdc(args),
            "LA_EIGENVAL" => linalg::la_eigenval(args),
            "LUDC" => linalg::ludc(args),
            "LUSOL" => linalg::lusol(args),
            // Additional linear algebra (Phase 12)
            "LA_EIGENVEC" => linalg::la_eigenvec(args),
            "LA_LINEAR_EQUATION" => linalg::la_linear_equation(args),
            "LA_LEAST_SQUARES" => linalg::la_least_squares(args),
            "LA_CHOLESKY" | "CHOLESKY" => linalg::la_cholesky(args),
            "LA_TRIDC" | "TRIDC" => linalg::la_tridc(args),
            "QR" => linalg::qr(args),
            "RANK" | "MATRIX_RANK" => linalg::matrix_rank(args),
            "CRAMER" => linalg::cramer(args),
            "MATRIX_MULTIPLY" => linalg::matrix_multiply(args),
            "COND" => linalg::cond(args),
            "PINV" => linalg::pinv(args),

            // Signal processing functions
            "A_CORRELATE" => signal::a_correlate(args),
            "C_CORRELATE" => signal::c_correlate(args),
            "DIGITAL_FILTER" => signal::digital_filter(args),
            "HILBERT" => signal::hilbert(args),
            "MEDIAN_FILTER" => signal::median_filter(args),
            // Additional signal processing (Phase 11)
            "FFT_2D" | "FFT2" => signal::fft_2d(args),
            "HANNING" => signal::hanning(args),
            "HAMMING" => signal::hamming(args),
            "BLACKMAN" => signal::blackman(args),
            "BUTTERWORTH" => signal::butterworth(args),
            "SAVGOL" => signal::savgol(args),
            "LEEFILT" => signal::leefilt(args),
            "WV_HAAR" => signal::wv_haar(args),
            "WV_IHAAR" => signal::wv_ihaar(args),
            "WV_DWT" => signal::wv_dwt(args),
            "POWER_SPECTRUM" => signal::power_spectrum(args),

            // Image processing functions
            "CONVOL" => image::convol(args),
            "DILATE" => image::dilate(args),
            "ERODE" => image::erode(args),
            "SOBEL" => image::sobel(args),
            "ROBERTS" => image::roberts(args),
            "PREWITT" => image::prewitt(args),
            "GAUSSIAN_FILTER" => image::gaussian_filter(args),
            "THRESHOLD" => image::threshold(args),
            // Additional image processing (Phase 13)
            "CANNY" => image::canny(args),
            "HOUGH" => image::hough(args),
            "RADON" => image::radon(args),
            "WATERSHED" => image::watershed(args),
            "LABEL_REGION" => image::label_region(args),
            "MORPH_OPEN" => image::morph_open(args),
            "MORPH_CLOSE" => image::morph_close(args),
            "HIST_EQUAL" => image::hist_equal(args),
            "EDGE_DOG" => image::edge_dog(args),
            "LAPLACIAN" => image::laplacian(args),
            "MEDIAN_2D" => image::median_2d(args),

            // Python integration functions (requires "python" feature)
            #[cfg(feature = "python")]
            "PYTHON_IMPORT" => python::python_import(args),
            #[cfg(feature = "python")]
            "PYTHON_CALL" => python::python_call(args),
            #[cfg(feature = "python")]
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

            // Additional ML Functions (Tier 1 & 2)
            "XDLML_AVERAGEPOOLING2D" => ml::xdlml_averagepooling2d(args),
            "XDLML_CONFUSIONMATRIX" => ml::xdlml_confusionmatrix(args),
            "XDLML_LINEARREGRESSION" => ml::xdlml_linearregression(args),
            "XDLML_LOGISTICREGRESSION" => ml::xdlml_logisticregression(args),
            "XDLML_PCA" => ml::xdlml_pca(args),
            "XDLML_NAIVEBAYES" => ml::xdlml_naivebayes(args),
            "XDLML_ONEHOTENCODER" => ml::xdlml_onehotencoder(args),
            "XDLML_LABELENCODER" => ml::xdlml_labelencoder(args),
            "XDLML_LAYERNORMALIZATION" => ml::xdlml_layernormalization(args),

            // =========================================================================
            // Polars DataFrame functions (requires "dataframes" feature)
            // =========================================================================
            #[cfg(feature = "dataframes")]
            "DF_READ_CSV" => polars_df::df_read_csv(args),
            #[cfg(feature = "dataframes")]
            "DF_READ_PARQUET" => polars_df::df_read_parquet(args),
            #[cfg(feature = "dataframes")]
            "DF_READ_JSON" => polars_df::df_read_json(args),
            #[cfg(feature = "dataframes")]
            "DF_CREATE" => polars_df::df_create(args),
            #[cfg(feature = "dataframes")]
            "DF_WRITE_CSV" => polars_df::df_write_csv(args),
            #[cfg(feature = "dataframes")]
            "DF_WRITE_PARQUET" => polars_df::df_write_parquet(args),
            #[cfg(feature = "dataframes")]
            "DF_HEAD" => polars_df::df_head(args),
            #[cfg(feature = "dataframes")]
            "DF_TAIL" => polars_df::df_tail(args),
            #[cfg(feature = "dataframes")]
            "DF_SELECT" => polars_df::df_select(args),
            #[cfg(feature = "dataframes")]
            "DF_FILTER" => polars_df::df_filter(args),
            #[cfg(feature = "dataframes")]
            "DF_SORT" => polars_df::df_sort(args),
            #[cfg(feature = "dataframes")]
            "DF_GROUPBY" => polars_df::df_groupby(args),
            #[cfg(feature = "dataframes")]
            "DF_JOIN" => polars_df::df_join(args),
            #[cfg(feature = "dataframes")]
            "DF_SHAPE" => polars_df::df_shape(args),
            #[cfg(feature = "dataframes")]
            "DF_COLUMNS" => polars_df::df_columns(args),
            #[cfg(feature = "dataframes")]
            "DF_DTYPES" => polars_df::df_dtypes(args),
            #[cfg(feature = "dataframes")]
            "DF_DESCRIBE" => polars_df::df_describe(args),
            #[cfg(feature = "dataframes")]
            "DF_PRINT" => polars_df::df_print(args),
            #[cfg(feature = "dataframes")]
            "DF_TO_ARRAY" => polars_df::df_to_array(args),
            #[cfg(feature = "dataframes")]
            "DF_DROP" => polars_df::df_drop(args),

            // =========================================================================
            // Linfa ML functions (requires "ml" feature)
            // =========================================================================
            #[cfg(feature = "ml")]
            "ML_KMEANS_FIT" => linfa_ml::ml_kmeans_fit(args),
            #[cfg(feature = "ml")]
            "ML_KMEANS_PREDICT" => linfa_ml::ml_kmeans_predict(args),
            #[cfg(feature = "ml")]
            "ML_KMEANS_CENTROIDS" => linfa_ml::ml_kmeans_centroids(args),
            #[cfg(feature = "ml")]
            "ML_LINEAR_FIT" => linfa_ml::ml_linear_fit(args),
            #[cfg(feature = "ml")]
            "ML_LINEAR_PREDICT" => linfa_ml::ml_linear_predict(args),
            #[cfg(feature = "ml")]
            "ML_LINEAR_COEFFICIENTS" => linfa_ml::ml_linear_coefficients(args),
            #[cfg(feature = "ml")]
            "ML_LINEAR_INTERCEPT" => linfa_ml::ml_linear_intercept(args),
            #[cfg(feature = "ml")]
            "ML_LOGISTIC_FIT" => linfa_ml::ml_logistic_fit(args),
            #[cfg(feature = "ml")]
            "ML_LOGISTIC_PREDICT" => linfa_ml::ml_logistic_predict(args),
            #[cfg(feature = "ml")]
            "ML_PCA_FIT" => linfa_ml::ml_pca_fit(args),
            #[cfg(feature = "ml")]
            "ML_PCA_TRANSFORM" => linfa_ml::ml_pca_transform(args),
            #[cfg(feature = "ml")]
            "ML_PCA_COMPONENTS" => linfa_ml::ml_pca_components(args),
            #[cfg(feature = "ml")]
            "ML_PCA_VARIANCE" => linfa_ml::ml_pca_variance(args),
            #[cfg(feature = "ml")]
            "ML_TRAIN_TEST_SPLIT" => linfa_ml::ml_train_test_split(args),
            #[cfg(feature = "ml")]
            "ML_ACCURACY" => linfa_ml::ml_accuracy(args),
            #[cfg(feature = "ml")]
            "ML_MSE" => linfa_ml::ml_mse(args),
            #[cfg(feature = "ml")]
            "ML_R2_SCORE" => linfa_ml::ml_r2_score(args),
            #[cfg(feature = "ml")]
            "ML_DROP_MODEL" => linfa_ml::ml_drop_model(args),

            // =========================================================================
            // RustPython functions (requires "rustpython" feature)
            // =========================================================================
            #[cfg(feature = "rustpython")]
            "RUSTPY_EXEC" => rustpython_interp::rustpy_exec(args),
            #[cfg(feature = "rustpython")]
            "RUSTPY_EVAL" => rustpython_interp::rustpy_eval(args),
            #[cfg(feature = "rustpython")]
            "RUSTPY_CALL" => rustpython_interp::rustpy_call(args),
            #[cfg(feature = "rustpython")]
            "RUSTPY_IMPORT" => rustpython_interp::rustpy_import(args),
            #[cfg(feature = "rustpython")]
            "RUSTPY_VERSION" => rustpython_interp::rustpy_version(args),
            #[cfg(feature = "rustpython")]
            "RUSTPY_STDLIB" => rustpython_interp::rustpy_stdlib(args),

            // =========================================================================
            // AMP (Accelerated Math Processing) functions
            // =========================================================================
            "AMP_INFO" => amp::amp_info(args),
            "AMP_BACKEND" => amp::amp_backend(args),
            "AMP_STATS" => amp::amp_stats(args),
            "AMP_RESET_STATS" => amp::amp_reset_stats(args),
            "AMP_BENCHMARK" => amp::amp_benchmark(args),
            "AMP_GPU_AVAILABLE" => amp::amp_gpu_available(args),
            // SIMD operations
            "AMP_SIMD_ADD" => amp::amp_simd_add(args),
            "AMP_SIMD_SUB" => amp::amp_simd_sub(args),
            "AMP_SIMD_MUL" => amp::amp_simd_mul(args),
            "AMP_SIMD_DIV" => amp::amp_simd_div(args),
            "AMP_SIMD_SQRT" => amp::amp_simd_sqrt(args),
            "AMP_SIMD_SUM" => amp::amp_simd_sum(args),
            "AMP_SIMD_MAX" => amp::amp_simd_max(args),
            "AMP_SIMD_MIN" => amp::amp_simd_min(args),
            "AMP_SIMD_DOT" => amp::amp_simd_dot(args),
            "AMP_SIMD_MATMUL" => amp::amp_simd_matmul(args),
            // GPU operations
            "AMP_GPU_ADD" => amp::amp_gpu_add(args),
            "AMP_GPU_MATMUL" => amp::amp_gpu_matmul(args),

            // =========================================================================
            // Object System Functions
            // =========================================================================
            "OBJ_NEW" => data_structures::obj_new(args),
            "OBJ_VALID" => data_structures::obj_valid(args),
            "OBJ_DESTROY" => data_structures::obj_destroy(args),
            "OBJ_CLASS" => data_structures::obj_class(args),
            "OBJ_ISA" => data_structures::obj_isa(args),
            "OBJ_HASMETHOD" => data_structures::obj_hasmethod(args),
            "OBJ_PARENT" => data_structures::obj_parent(args),
            "CALL_METHOD" => data_structures::call_method(args),
            "SETPROPERTY" => data_structures::setproperty(args),
            "GETPROPERTY" => data_structures::getproperty(args),
            "PTR_NEW" => data_structures::ptr_new(args),
            "PTR_VALID" => data_structures::ptr_valid(args),
            "PTR_FREE" => data_structures::ptr_free(args),
            "PTR_DEREF" => data_structures::ptr_deref(args),
            // Data structure functions
            "LIST" => data_structures::list(args),
            "LIST_ADD" => data_structures::list_add(args),
            "LIST_COUNT" => data_structures::list_count(args),
            "ORDEREDHASH" => data_structures::orderedhash(args),
            "DICTIONARY" => data_structures::dictionary(args),
            "CREATE_STRUCT" => data_structures::create_struct(args),
            "STRUCT_ASSIGN" => data_structures::struct_assign(args),
            "HEAP_GC" => data_structures::heap_gc(args),
            "HEAP_FREE" => data_structures::heap_free(args),

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
