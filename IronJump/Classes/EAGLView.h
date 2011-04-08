//
//  EAGLView.h
//  IronJump
//
//  Created by Filip Kunc on 4/18/10.
//  For license see LICENSE.TXT
//

#import <UIKit/UIKit.h>
#import <QuartzCore/QuartzCore.h>
#import <OpenGLES/EAGL.h>
#import <OpenGLES/EAGLDrawable.h>
#import <OpenGLES/ES1/gl.h>
#import <OpenGLES/ES1/glext.h>
#import "FPMath.h"
#import "FPTextureAtlas.h"
#import "FPFont.h"
#import "FPGame.h"
#import "FPExit.h"
#import "FPStage.h"

// This class wraps the CAEAGLLayer from CoreAnimation into a convenient UIView subclass.
// The view content is basically an EAGL surface you render your OpenGL scene into.
// Note that setting the view non-opaque will only work if the EAGL surface has an alpha channel.
@interface EAGLView : UIView <UIAccelerometerDelegate>
{    
    EAGLContext *context;
    
    // The pixel dimensions of the CAEAGLLayer
    GLint backingWidth;
    GLint backingHeight;
    
    // The OpenGL ES names for the framebuffer and renderbuffer used to render to this view
    GLuint defaultFramebuffer, colorRenderbuffer;
	
	FPTexture *win;
	float winAnimation;
	BOOL victory;
	
	FPGame *game;
    id<FPGameObject> exit;
	
	int nextLevelCounter;

    BOOL animating;
    BOOL displayLinkSupported;
    NSInteger animationFrameInterval;
    // Use of the CADisplayLink class is the preferred method for controlling your animation timing.
    // CADisplayLink will link to the main display and fire every vsync when added to a given run-loop.
    // The NSTimer class is used only as fallback when running on a pre 3.1 device where CADisplayLink
    // isn't available.
    id displayLink;
    NSTimer *animationTimer;
	float lastAcceleration;
}

@property (readonly, nonatomic, getter=isAnimating) BOOL animating;
@property (nonatomic) NSInteger animationFrameInterval;

- (void)startAnimation;
- (void)stopAnimation;
- (void)drawView:(id)sender;
- (void)resetGame;
- (void)nextLevelIfNeeded;
- (void)resetIfNeeded;
- (void)update;
- (void)render;

@end
