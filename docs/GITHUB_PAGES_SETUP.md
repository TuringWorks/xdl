# GitHub Pages Setup Guide

This guide will help you enable and configure GitHub Pages for the XDL documentation.

## Quick Setup

### 1. Enable GitHub Pages

1. Go to your repository on GitHub: `https://github.com/ravituringworks/xdl`
2. Click on **Settings** tab
3. In the left sidebar, click on **Pages**
4. Under **Build and deployment**:
   - **Source**: Select "GitHub Actions"
5. Click **Save**

### 2. Verify Deployment

After pushing changes to the master branch:

1. Go to the **Actions** tab in your repository
2. You should see a workflow run called "Deploy GitHub Pages"
3. Wait for the workflow to complete (usually takes 1-2 minutes)
4. Once complete, your documentation will be available at:
   - `https://ravituringworks.github.io/xdl/`

## What's Been Configured

### Documentation Structure

The documentation is set up in the `/docs` folder with:

- **index.md**: Main landing page with organized links to all documentation
- **_config.yml**: Jekyll configuration for GitHub Pages
- **90+ documentation files**: All your existing documentation is linked and organized

### Jekyll Theme

The documentation uses the **Cayman** theme, which provides:

- Clean, professional design
- Responsive layout for mobile devices
- Syntax highlighting for code blocks
- Easy navigation

### GitHub Actions Workflow

A workflow has been created at `.github/workflows/pages.yml` that:

- Automatically builds and deploys documentation on every push to master
- Uses Jekyll to process the markdown files
- Deploys to GitHub Pages

## Customization

### Changing the Theme

To change the theme, edit `docs/_config.yml`:

```yaml
theme: jekyll-theme-minimal  # or other supported themes
```

Supported themes:

- `jekyll-theme-cayman` (current)
- `jekyll-theme-minimal`
- `jekyll-theme-slate`
- `jekyll-theme-architect`
- `jekyll-theme-tactile`
- `jekyll-theme-dinky`

### Adding Navigation

Edit the navigation section in `docs/_config.yml`:

```yaml
navigation:
  - title: Home
    url: /
  - title: Custom Page
    url: /custom-page.html
```

### Customizing the Landing Page

Edit `docs/index.md` to modify the main landing page content.

## Documentation Organization

The documentation is organized into these main sections:

1. **Getting Started**: Quick start guides and installation
2. **Core Features**: Language, parser, arrays, and control flow
3. **Graphics & Visualization**: 2D, 3D, and advanced visualization
4. **GPU Acceleration**: GPU compute and performance
5. **Compatibility**: IDL/GDL and MATLAB compatibility
6. **Advanced Topics**: ML, Python integration, examples
7. **Development**: Build guides and validation

## Troubleshooting

### Pages Not Showing Up

1. Check the Actions tab for any failed workflow runs
2. Ensure GitHub Pages is enabled in Settings > Pages
3. Verify the source is set to "GitHub Actions"
4. Wait a few minutes after deployment completes

### 404 Errors on Documentation Links

1. Ensure all linked files exist in the `docs/` folder
2. Check that file names match the links (case-sensitive)
3. Links use `.html` extension even though files are `.md`

### Build Failures

1. Check the Actions tab for error messages
2. Verify Jekyll configuration in `_config.yml` is valid
3. Ensure all markdown files have valid formatting

## Local Testing

To test the documentation locally before deploying:

```bash
# Install Jekyll
gem install bundler jekyll

# Navigate to docs folder
cd docs

# Create Gemfile
cat > Gemfile << EOF
source 'https://rubygems.org'
gem 'github-pages', group: :jekyll_plugins
EOF

# Install dependencies
bundle install

# Serve locally
bundle exec jekyll serve

# Open in browser
# http://localhost:4000
```

## Additional Resources

- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [Jekyll Documentation](https://jekyllrb.com/docs/)
- [Supported Themes](https://pages.github.com/themes/)
- [Jekyll on GitHub Pages](https://docs.github.com/en/pages/setting-up-a-github-pages-site-with-jekyll)

## Next Steps

1. Enable GitHub Pages in repository settings
2. Push changes to trigger the first deployment
3. Visit your documentation site once deployed
4. Customize the theme and content as needed
5. Add more documentation pages as your project grows
