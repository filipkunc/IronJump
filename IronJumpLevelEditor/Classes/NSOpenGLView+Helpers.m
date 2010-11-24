//
//  NSOpenGLView+Helpers.m
//  MacRubyGame
//
//  Created by Filip Kunc on 3/25/10.
//  All rights reserved.
//

#import "NSOpenGLView+Helpers.h"

NSOpenGLPixelFormat *globalPixelFormat = nil;
NSOpenGLContext *globalGLContext = nil;

@implementation NSOpenGLView(Helpers)

+ (NSOpenGLPixelFormat *)sharedPixelFormat
{
	if (!globalPixelFormat)
	{
		NSOpenGLPixelFormatAttribute attribs[] = 
		{
			NSOpenGLPFAAccelerated,
			NSOpenGLPFADoubleBuffer,
			NSOpenGLPFAColorSize, 1,
			NSOpenGLPFADepthSize, 1,
			0 
		};
		
		globalPixelFormat = [[NSOpenGLPixelFormat alloc] initWithAttributes:attribs];
	}
	return globalPixelFormat;
}

+ (NSOpenGLContext *)sharedContext
{
	if (!globalGLContext)
	{
		globalGLContext = [[NSOpenGLContext alloc] initWithFormat:[NSOpenGLView sharedPixelFormat]
													 shareContext:nil];
	}
	return globalGLContext;
}

- (void)setupSharedContext
{
	[self clearGLContext];
	NSOpenGLContext *context = [[NSOpenGLContext alloc] initWithFormat:[NSOpenGLView sharedPixelFormat]
														  shareContext:[NSOpenGLView sharedContext]];
	GLint swapInt = 1;
    [context setValues:&swapInt forParameter:NSOpenGLCPSwapInterval];
	
	[self setOpenGLContext:context];
	[context release];
	[[self openGLContext] makeCurrentContext];
}

- (CGSize)reshapeFlippedOrtho2D
{
	NSRect rect = [self visibleRect];
	
	glViewport(0, 0, rect.size.width, rect.size.height);
	
	glMatrixMode(GL_PROJECTION);
	glLoadIdentity();
	glOrtho(0, rect.size.width, 0, rect.size.height, -1.0f, 1.0f);
	
	glMatrixMode(GL_MODELVIEW);
	glLoadIdentity();
	
	glTranslatef(-rect.origin.x, rect.origin.y + rect.size.height, 0);
	glScalef(1, -1, 1);
	
	return CGSizeMake(rect.size.width, rect.size.height);
}

- (CGPoint)locationFromNSEvent:(NSEvent *)e
{
	NSPoint point = [self convertPoint:[e locationInWindow] fromView:nil];
	return CGPointMake(point.x, point.y);
}

- (CGRect)boundsCG
{
	NSRect bounds = [self bounds];
	return CGRectMake(bounds.origin.x, bounds.origin.y, bounds.size.width, bounds.size.height);
}

@end
