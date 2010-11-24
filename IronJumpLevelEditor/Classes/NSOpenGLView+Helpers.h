//
//  NSOpenGLView+Helpers.h
//  MacRubyGame
//
//  Created by Filip Kunc on 3/25/10.
//  All rights reserved.
//

#import <Cocoa/Cocoa.h>
#import <OpenGL/OpenGL.h>
#import <OpenGL/glu.h>

#import "FPGraphics.h"

@interface NSOpenGLView(Helpers)

+ (NSOpenGLPixelFormat *)sharedPixelFormat;
+ (NSOpenGLContext *)sharedContext;
- (void)setupSharedContext;
- (CGSize)reshapeFlippedOrtho2D;
- (CGPoint)locationFromNSEvent:(NSEvent *)e;
- (CGRect)boundsCG;

@end